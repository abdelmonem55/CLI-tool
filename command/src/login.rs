// func init() {
// loginCmd.Flags().StringVarP(&gateway, "gateway", "g", defaultGateway, "Gateway URL starting with http(s)://")
// loginCmd.Flags().StringVarP(&username, "username", "u", "admin", "Gateway username")
// loginCmd.Flags().StringVarP(&password, "password", "p", "", "Gateway password")
// loginCmd.Flags().BoolVarP(&passwordStdin, "password-stdin", "s", false, "Reads the gateway password from stdin")
// loginCmd.Flags().BoolVar(&tlsInsecure, "tls-no-verify", false, "Disable TLS validation")
// loginCmd.Flags().Duration("timeout", time.Second*5, "Override the timeout for this API call")
//
// faasCmd.AddCommand(loginCmd)
// }
//
// var loginCmd = &cobra.Command{
// Use:   `login [--username admin|USERNAME] [--password PASSWORD] [--gateway GATEWAY_URL] [--tls-no-verify]`,
// Short: "Log in to OpenFaaS gateway",
// Long:  "Log in to OpenFaaS gateway.\nIf no gateway is specified, the default value will be used.",
// Example: `  cat ~/faas_pass.txt | faas-cli login -u user --password-stdin
// echo $PASSWORD | faas-cli login -s  --gateway https://openfaas.mydomain.com
// faas-cli login -u user -p password`,
// RunE: runLogin,
// }

use crate::error::{check_tls_insecure, NOT_TLS_WARN};
use crate::faas::DEFAULT_GATEWAY;
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::{CommandAppend, State};
use clap::{App, Arg, ArgMatches, SubCommand};
use config::config_file::{
    decode_auth, encode_auth, lookup_auth_config, update_auth_config, BASIC_AUTH_TYPE,
};
use proxy::proxy::make_http_client;
use reqwest::StatusCode;
use reqwest::{Method, Url};
use std::io::Read;
use std::time::Duration;

pub(crate) struct Login;

impl CommandAppend for Login {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("login")
                .about(r#"Log in to OpenFaaS gateway.\nIf no gateway is specified, the default value will be used.",
Example: `  cat ~/faas_pass.txt | faas-cli login -u user --password-stdin
echo $PASSWORD | faas-cli login -s  --gateway https://openfaas.mydomain.com
 faas-cli login -u user -p password`,"#)
                // .arg_from_usage("<name> 'function name'")
                //.arg_from_usage("-g, --gateway [gateway]")

                .arg(
                    Arg::with_name("username")
                        .long("username")
                        .short("u")
                        .default_value("admin")
                        .takes_value(true)
                        .global(true)
                        .help("Gateway username"),
                )
                // .arg(
                //     Arg::with_name("gateway")
                //         .long("gateway")
                //         .short("g")
                //         .default_value(DEFAULT_GATEWAY)
                //         .takes_value(true)
                //         .global(true)
                //         .help("Gateway URL starting with http(s)://"),
                // )
                .arg(
                    Arg::with_name("timeout")
                        .long("timeout")
                        .default_value("5 sec")
                        .takes_value(true)
                        .global(true)
                        .help("Override the timeout for this API call in Duration using units defined \
                        in 'https://www.freedesktop.org/software/systemd/man/systemd.time.html#Parsing%20Time%20Spans' \
             --timeout VALUE
             Example: --timeout 5sec   or --timeout '5 sec'  'will set timeout to 5 seconds'"),
                )
                .args_from_usage(
                    "-p ,--password [password] 'Gateway password'
                        --tls-no-verify 'Disable TLS validation'
                        --password-stdin  'Reads the gateway password from stdin'
            ",
                )
        );
        app
    }
}

impl Login {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(l_args) = args.subcommand_matches("login") {
            let gateway = args.value_of("gateway").ok_or(State::Custom(format!(
                "you must set gateway using \
             --gateway, -g http://host"
            )))?;
            let timeout = l_args.value_of("timeout").ok_or(State::Custom(format!(
                "you must set timeout in duration using systemd units
                 in https://www.freedesktop.org/software/systemd/man/systemd.time.html#Parsing%20Time%20Spans \
             --timeout VALUE
             Example: --timeout 5sec   or --timeout '5 sec'  'will set timeout to 5 seconds'
             "
            )))?;
            let username = l_args.value_of("username").ok_or(State::Custom(format!(
                "you must set username using \
             --username VALUE"
            )))?;
            let password_stdin_presented = l_args.is_present("password-stdin");
            let tls_insecure = l_args.is_present("tls-no-verify");
            let mut password = String::new();

            if let Some(pass) = l_args.value_of("password") {
                println!("WARNING! Using --password is insecure, consider using: cat ~/faas_pass.txt | faas-cli login -u user --password-stdin");
                if password_stdin_presented {
                    return Err(State::Custom(
                        "--password and --password-stdin are mutually exclusive".into(),
                    ));
                }
                password = pass.to_string();
            }

            if password_stdin_presented {
                let mut password_stdin = Vec::new();
                std::io::stdin()
                    .read_to_end(&mut password_stdin)
                    .map_err(|e| State::Custom(e.to_string()))?;
                let pass = std::str::from_utf8(password_stdin.as_slice())
                    .map_err(|e| State::Custom(e.to_string()))?;
                password = pass.to_string();

                //password = strings.TrimSpace(string(passwordStdin))
            }
            password = password.trim().to_string();
            if password.is_empty() {
                return Err(State::Custom(
                    "must provide a non-empty password via --password or --password-stdin"
                        .to_string(),
                ));
            }

            println!("Calling the OpenFaaS server to validate the credentials...");
            let openfass_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or(String::new());
            let gateway = get_gateway_url(gateway, DEFAULT_GATEWAY, "", openfass_url.as_str());

            // let timeout: u64 = timeout
            //     .parse()
            //     .map_err(|_| State::Custom("can't parse time out".to_string()))?;
            let timeout = parse_duration::parse(timeout)
                .map_err(|e| State::Custom(format!("{} , you must set timeout in duration using systemd units
                 in https://www.freedesktop.org/software/systemd/man/systemd.time.html#Parsing%20Time%20Spans \
             --timeout VALUE
             Example: --timeout 5sec   or --timeout '5 sec'  'will set timeout to 5 seconds'",e)))?;

            validate_login(
                gateway.as_str(),
                username,
                password.as_str(),
                Some(timeout),
                tls_insecure,
            )
            .await?;

            let token = encode_auth(username, password.as_str());

            update_auth_config(gateway.as_str(), token.as_str(), BASIC_AUTH_TYPE.into())?;

            let auth_config = lookup_auth_config(gateway.as_str())?;

            let (user, _pass) = decode_auth(auth_config.token.as_str())?;

            println!("credentials saved for {} {}", user, gateway);
            Err(State::Matched)
        } else {
            //todo investigate the output
            Ok(())
        }
    }
}

async fn validate_login(
    gateway: &str,
    user: &str,
    pass: &str,
    timeout: Option<Duration>,
    insecure_tls: bool,
) -> crate::Result {
    if !check_tls_insecure(gateway, insecure_tls).is_empty() {
        println!("{}", NOT_TLS_WARN)
    }

    let client = make_http_client(timeout, insecure_tls)?;

    let url = format!("{}/system/functions", gateway.trim_end_matches('/'));
    let url = Url::parse(url.as_str()).map_err(|e| State::Custom(e.to_string()))?;
    let req = client.request(Method::GET, url);

    let req = req
        .basic_auth(user, Some(pass))
        .build()
        .map_err(|e| State::Custom(e.to_string()))?;

    let res = client.execute(req).await.map_err(|e| {
        State::Custom(format!(
            "cannot connect to OpenFaaS on URL: {}. {}",
            gateway, e
        ))
    })?;

    match res.status() {
        StatusCode::OK => Ok(()),
        StatusCode::UNAUTHORIZED => Err(State::Custom(
            "unable to login, either username or password is incorrect".to_string(),
        )),
        status => {
            let body = res.text().await;
            if let Ok(body) = body {
                Err(State::Custom(format!(
                    "server returned unexpected status code: {} - {}",
                    status, body
                )))
            } else {
                Ok(())
            }
        }
    }
}
