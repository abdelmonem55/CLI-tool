#![allow(unused)]
use crate::faas::DEFAULT_GATEWAY;
use crate::{CommandAppend, State};
use chrono::Timelike;
use clap::{App, ArgMatches, SubCommand};
use config::config_file::{update_auth_config, OAUTH_2AUTH_TYPE};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use reqwest::{Method, StatusCode, Url};
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use utility::{Error, Result};

#[derive(Debug, Clone)]
struct Data {
    gateway: String,
    tx: tokio::sync::mpsc::Sender<()>,
}

static mut GATEWAY: String = String::new();
const BUILD_CAPTURE_FRAGMENT: &str = r#"<html>
<head>
<title>OpenFaaS CLI Authorization flow</title>
<script>
	var xhttp = new XMLHttpRequest();
	xhttp.onreadystatechange = function() {
		if (this.readyState == 4 && this.status == 200) {
			console.log(xhttp.responseText)
		}
	};
	xhttp.open("GET", "/oauth2/callback?fragment="+document.location.hash.slice(1), true);
	xhttp.send();
</script>
</head>
<body>
 Authorization flow complete. Please close this browser window.
</body>
</html>"#;

pub(crate) struct Auth;

#[derive(serde::Serialize, Debug)]
struct ClientCredentialsReq<'s> {
    pub client_id: &'s str,     // `json:"client_id"`
    pub client_secret: &'s str, // `json:"client_secret"`
    pub audience: &'s str,      // `json:"audience"`
    pub grant_type: &'s str,    // `json:"grant_type"`
}

#[derive(serde::Deserialize, Debug)]
struct ClientCredentialsToken<'s> {
    #[serde(default)]
    pub access_token: &'s str, // `json:"access_token"`
    #[serde(default)]
    pub scope: &'s str, // `json:"scope"`
    pub expires_in: Option<i32>, //`json:"expires_in"`
    #[serde(default)]
    pub token_type: &'s str, // `json:"token_type"`
}

impl CommandAppend for Auth {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("auth")
                .about(r#"Generate and save the registry authentication file"#)
                .args_from_usage("
                    --auth-url         [auth-url]                      'OAuth2 Authorize URL i.e. http://idp/oauth/authorize'
                    --client-id        [client-id]                      'OAuth2 client_id'
                    --listen-port      [listen-port]                     'OAuth2 local port for receiving cookie'
                    --audience         [audience]                        'OAuth2 audience'
                    --launch-browse                                      'Launch browser for OAuth2 redirect'
                    --redirect-host    [redirect-host]                           'Host for OAuth2 redirection in the implicit flow including URL scheme'
                    --scope            [scope]                          'scope for OAuth2 flow - i.e. \"openid profile\"'
                    --grant            [grant]                          'grant for OAuth2 flow - either implicit, implicit-id or client_credentials'
                    --client-secret    [client-secret]                 'OAuth2 client_secret, for use with client_credentials grant'
             ",
                )
        );
        app
    }
}

impl Auth {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(a_args) = args.subcommand_matches("auth") {
            let gateway = args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
            unsafe {
                GATEWAY = gateway.to_string();
            }
            let auth_url = a_args.value_of("auth-url").unwrap_or_default();
            let client_id = a_args.value_of("client-id").unwrap_or_default();
            let listen_port = a_args.value_of("listen-port").unwrap_or("31111");
            let audience = a_args.value_of("audience").unwrap_or_default();
            let redirect_host = a_args
                .value_of("redirect-host")
                .unwrap_or("http://127.0.0.1");
            let scope = a_args.value_of("scope").unwrap_or("openid profile");
            let grant = a_args.value_of("grant").unwrap_or("implicit");
            let client_secret = a_args.value_of("client-secret").unwrap_or_default();
            //todo check this
            let launch_browser = true; //a_args.is_present("launch-browser");
            let listen_port = listen_port.parse().map_err(|_| {
                State::Custom(format!("can't parse {} as u16 listen port", listen_port))
            })?;

            check_values(auth_url, client_id)?;
            let mut data = AuthData {
                gateway,
                grant,
                listen_port,
                redirect_host,
                auth_url,
                scope,
                audience,
                client_id,
                //token: "",
                launch_browser,
            };

            if grant == "implicit" {
                data.grant = "token";
                auth_implicit(&data).await?;
            } else if grant == "implicit-id" {
                data.grant = "id_token";
                auth_implicit(&data).await?;
            } else if grant == "client_credentials" {
                auth_client_credentials(
                    client_id,
                    client_secret,
                    audience,
                    grant,
                    auth_url,
                    gateway,
                )
                .await?;
            } else {
                return Err(State::Custom(format!(
                    "invalid grant {} please enter one of these grants 'implicit-id' ,\
                 'implicit-id', 'client_credentials or implicit'",
                    grant
                )));
            }

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

fn check_values(auth_url: &str, client_id: &str) -> Result<()> {
    if auth_url.is_empty() {
        return Err(Error::Custom(
            "--auth-url is required and must be a valid OIDC URL".to_string(),
        ));
    }

    let url = url::Url::parse(auth_url)
        .map_err(|e| Error::Custom(format!("--auth-url is an invalid URL: {}", e)))?;

    if url.scheme() != "http" && url.scheme() != "https" {
        return Err(Error::Custom(format!(
            "--auth-url is an invalid URL: {}",
            url.as_str()
        )));
    }

    if client_id.is_empty() {
        return Err(Error::Custom("--client-id is required".to_string()));
    }

    Ok(())
}

struct AuthData<'s> {
    gateway: &'s str,
    grant: &'s str,
    listen_port: u16,
    redirect_host: &'s str,
    auth_url: &'s str,
    scope: &'s str,
    audience: &'s str,
    client_id: &'s str,
    //token: &'s str,
    launch_browser: bool,
}
async fn auth_implicit(auth_data: &AuthData<'_>) -> Result<()> {
    // We'll bind to 127.0.0.1:3000
    //http://127.0.0.1/
    let addr = ([127, 0, 0, 1], auth_data.listen_port).into();

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.

    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(1);

    let data = Data {
        gateway: auth_data.gateway.to_string(),
        tx: tx.clone(),
    };
    let data = Arc::new(data);

    let make_service = make_service_fn(move |_conn| {
        let data = data.clone();
        let tx = tx.clone();
        async move {
            Ok::<_, Error>(service_fn(move |req: Request<Body>| {
                let data = data.clone();
                let tx = tx.clone();

                //Ok::<_, Infallible>(request_handler(req,data))
                async move {
                    match request_handler(req, data).await {
                        Ok(res) => Ok(res),
                        Err(e) => {
                            tx.send(())
                                .await
                                .map_err(|e| Error::Custom(e.to_string()))?;
                            Err(e)
                        }
                    }
                }
            }))
        }
    });

    //Server::bind(&addr).serve(make_service).await.unwrap();
    let server = Server::bind(&addr).serve(make_service);
    let graceful = server.with_graceful_shutdown(async move {
        let _ = rx.recv().await;
    });

    let handler = tokio::spawn(async move {
        println!("run the server");
        // Run this server until receive message to terminate!
        if let Err(e) = graceful.await {
            eprintln!("server error: {}", e);
        }
    });
    let mut auth_url_val =
        url::Url::parse(auth_data.auth_url).map_err(|e| Error::Custom(e.to_string()))?;

    {
        let mut q = auth_url_val.query_pairs_mut();

        // let req = client.request(Method::GET,)
        q.append_pair("response_mode", "fragment");
        q.append_pair("audience", auth_data.audience);
        q.append_pair("client_id", auth_data.client_id);

        let now = chrono::Local::now().timestamp_nanos().to_string();
        q.append_pair("nonce", now.as_str());
        let uri = make_redirect_url(auth_data.redirect_host, auth_data.listen_port)?;
        println!("redirect {}\n", uri);
        q.append_pair("redirect_uri", uri.as_str());
        q.append_pair("response_type", auth_data.grant);

        q.append_pair("scope", auth_data.scope);
        // let state = chrono::Local::now().timestamp_nanos().to_string();
        q.append_pair("state", now.as_str());
    }

    //let launch_uri = format!("https://abdelmonem.us.auth0.com/authorize?%26response_mode=fragment&audience=https%3A%2F%2Fabdelmonem.us.auth0.com%2Fapi%2Fv2%2F&client_id=0dFGCpXdzAzAKWejwYywI8hmVP7wTqsv&nonce=1625617096342424200&redirect_uri=http%3A%2F%2F127.0.0.1%3A31111%2Foauth%2Fcallback&response_type=id_token&scope=oidc+profile&state=1625617096342424200");
    let launch_uri = auth_url_val.to_string();
    println!("{}", auth_url_val);
    println!("Launching browser: {}\n", launch_uri);
    if auth_data.launch_browser {
        launch_url(launch_uri.as_str())
            .map_err(|_e| Error::Custom("unable to launch browser".to_string()))?;
    }
    //
    // <-context.Done()
    //
    // return nil
    handler.await.map_err(|e| Error::Custom(e.to_string()))?;
    Ok(())
}

async fn request_handler(req: Request<Body>, data: Arc<Data>) -> Result<Response<Body>> {
    // println!("in request ....................");

    let gateway = data.gateway.as_str();

    //   println!("{:#?}",req.uri());
    let url = format!(
        "http://example.com/{}",
        req.uri().to_string().trim_start_matches("/")
    );

    // let url = url::Url::parse(url.as_str())
    //     .map_err(|e| Error::Custom(e.to_string()))?;
    let url = reqwest::Url::parse(url.as_str()).map_err(|e| {
        let fmt = format!("error in receiving redirect url {}: is {:?}", url, e);
        println!("{}", fmt);
        Error::Custom(fmt)
    })?;
    let pairs: HashMap<_, _> = url.query_pairs().into_owned().collect();

    if let Some(v) = pairs.get("fragment") {
        //fmt.Println("v: ",v)
        let url = "http://example.com?".to_string() + v;
        let q = reqwest::Url::parse(url.as_str()).map_err(|e| {
            let fmt = format!(
                "unable to parse fragment response from browser redirect url {} is {:?}",
                url, e
            );
            println!("{}", fmt);
            Error::Custom(fmt)
        })?;

        let q: HashMap<_, _> = q.query_pairs().into_owned().collect();

        // log.Println("QueryString:", q)

        if let Some(token) = q.get("id_token") {
            update_auth_config(gateway, token, OAUTH_2AUTH_TYPE.into()).map_err(|e| {
                let fmt = format!("error while saving authentication token :{:?}", e);
                println!("{}", fmt);
                Error::Custom(fmt)
            })?;
            colour::green!("credentials saved for {}", gateway);

            print_example_token_usage(gateway, token.as_str());
        } else {
            println!("Unable to detect a valid id_token in URL fragment. Check your credentials or contact your administrator.\n{}",v);
        }

        data.tx.send(()).await.map_err(|e| {
            println!("{:?}", e);
            Error::Custom(e.to_string())
        })?;
    }

    Ok(Response::new(BUILD_CAPTURE_FRAGMENT.into()))
}

/// opens a URL with the default browser for Linux, MacOS or Windows.
fn launch_url(server_url: &str) -> Result<std::process::Output> {
    println!("url = {}", server_url);
    let mut command;
    #[cfg(target_os = "windows")]
    {
        let escaped = server_url.replace('&', "^&");
        let args = ["/c".to_string(), format!("start {}", escaped)];
        command = std::process::Command::new("cmd");
        command
            .args(&args)
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit());
    }
    #[cfg(target_os = "linux")]
    {
        let args = ["-c".to_string(), format!(r#"xdg-open "{}""#, server_url)];
        command = std::process::Command::new("sh");
        command.args(&args);
    }
    #[cfg(target_os = "darwin")]
    {
        let args = ["-c".to_string(), format!(r#"open "{}""#, server_url)];
        command = std::process::Command::new("sh");
        command.args(&args);
    }
    let out = command.output()?;
    Ok(out)
}

fn print_example_token_usage(gateway: &str, token: &str) {
    colour::green!(
        r#"Example usage:
# Use an explicit token\
faas-cli list --gateway "{}" --token "{}"

# Use the saved token
faas-cli list --gateway "{}""#,
        gateway,
        token,
        gateway
    );
}

async fn auth_client_credentials(
    client_id: &str,
    client_secret: &str,
    audience: &str,
    grant_type: &str,
    auth_url: &str,
    gateway: &str,
) -> Result<()> {
    let body = ClientCredentialsReq {
        client_id,
        client_secret,
        audience,
        grant_type,
    };
    let body_bytes = serde_json::to_string(&body)
        .map_err(|e| Error::Custom(format!("unable to serialize {:?} ,with error {}", body, e)))?;
    let url = reqwest::Url::parse(auth_url).map_err(|e| Error::Custom(e.to_string()))?;

    let client = reqwest::Client::new();
    let req = client
        .request(Method::POST, url)
        .header("Content-Type", "application/json")
        .body(body_bytes)
        .build()
        .map_err(|e| Error::Custom(format!("can't format a request: {:?}", e)))?;

    let res = client
        .execute(req)
        .await
        .map_err(|e| Error::Custom(format!("cannot POST to {} and the error {}", auth_url, e)))?;

    let status_code = res.status();
    let token_data = res
        .text()
        .await
        .map_err(|e| Error::Custom(format!("cannot read body and the error {}", e)))?;

    if status_code == StatusCode::OK {
        if !token_data.is_empty() {
            let token: ClientCredentialsToken =
                serde_json::from_str(token_data.as_str()).map_err(|e| {
                    Error::Custom(format!(
                        "can't deserialize {} ClientCredentialsToken to  and the error {}",
                        token_data, e
                    ))
                })?;

            update_auth_config(gateway, token.access_token, OAUTH_2AUTH_TYPE.into())?;

            colour::green!("credentials saved for {}", gateway);
            print_example_token_usage(gateway, token.access_token);
        }
        Ok(())
    } else {
        Err(Error::Custom(format!(
            "cannot authenticate, code: {}.\nResponse: {}",
            status_code, token_data
        )))
    }
}

fn make_redirect_url(host: &str, port: u16) -> Result<Url> {
    if host.starts_with("http://") || host.starts_with("https://") {
        let address = format!("{}:{}/oauth/callback", host, port);
        let url = Url::parse(address.as_str()).map_err(|e| {
            Error::Custom(format!(
                "can't parse {} as a url and the error: {}",
                address, e
            ))
        })?;
        Ok(url)
    } else {
        Err(Error::Custom(
            "a scheme is required for the URL for the host, i.e. http:// or https://".to_string(),
        ))
    }
}
