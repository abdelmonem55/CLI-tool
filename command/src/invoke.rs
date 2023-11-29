use crate::faas::{check_and_set_default_yaml, DEFAULT_GATEWAY};
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::{CommandAppend, State};
use clap::{App, Arg, ArgMatches, SubCommand};
use stack::schema::Services;
use stack::stack::parse_yaml_file;
use std::io::{Read, Write};
use utility::Error;

use hmac::{Hmac, Mac, NewMac};
use proxy::invoke::invoke_function;
use reqwest::Method;
use sha1::Sha1;

// Create alias for HMAC-SHA1
type HmacSha1 = Hmac<Sha1>;

pub(crate) struct Invoke;

impl CommandAppend for Invoke {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("invoke")
                .about(r#"Invokes an OpenFaaS function and reads from STDIN for the body of the request`,
	Example: `  faas-cli invoke echo --gateway https://host:port
  faas-cli invoke echo --gateway https://host:port --content-type application/json
  faas-cli invoke env --query repo=faas-cli --query org=openfaas
  faas-cli invoke env --header X-Ping-Url=http://request.bin/etc
  faas-cli invoke resize-img --async -H "X-Callback-Url=http://gateway:8080/function/send2slack" < image.png
  faas-cli invoke env -H X-Ping-Url=http://request.bin/etc
  faas-cli invoke flask --method GET --namespace dev
  faas-cli invoke env --sign X-GitHub-Event --key yoursecret`"#)
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

                .args_from_usage(
                    "
                          <NAME> 'Name of the deployed function'
                          --content-type [content-type] 'he content-type HTTP header such as application/json'
                          --query [query]               'pass query-string options like --query \"repo=faas-cli,org=openfaas\"'
                          -H ,--header [header]         'pass HTTP request header example: --header X-Ping-Url=http://request.bin/etc,name=my-name'
                          -a ,--async                   'Invoke the function asynchronously'
                          -m ,--method [method]          'pass HTTP request method'
                           --tls-no-verify              'Disable TLS validation'
                          --sign [sign]                  'name of HTTP request header to hold the signature'
                          --key [key]                    'key to be used to sign the request (must be used with --sign)'
                          -n ,--namespace  [namespace]             'Namespace of the deployed function'
            ",
                )
        );
        app
    }
}

impl Invoke {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(l_args) = args.subcommand_matches("invoke") {
            let regex = args.value_of("regex").unwrap_or("");
            let filter = args.value_of("filter").unwrap_or("");
            let yaml_file = args
                .value_of("yaml")
                .unwrap_or(check_and_set_default_yaml().unwrap_or_default());
            let envsubst = true; //args.is_present("envsubst");

            let gateway = args.value_of("gateway").ok_or(State::Custom(format!(
                "you must set gateway using \
             --gateway, -g http://host"
            )))?;
            let function_name = l_args.value_of("NAME").ok_or(State::Custom(format!(
                "you must set function name using \
             --name NAME"
            )))?;

            let key = l_args.value_of("key").unwrap_or("");
            let namespace = l_args.value_of("namespace").unwrap_or("");
            let method = l_args.value_of("method").unwrap_or("POST").to_uppercase();
            let sig_header = l_args.value_of("sign").unwrap_or("");
            let query: Vec<&str> = l_args.values_of("query").unwrap_or_default().collect();
            let mut headers: Vec<&str> = l_args.values_of("header").unwrap_or_default().collect();
            let content_type = l_args.value_of("content-type").unwrap_or("text/plain");
            let invoke_async = l_args.is_present("async");
            let tls_insecure = l_args.is_present("tls-no-verify");

            let method = Method::from_bytes(method.as_ref())
                .map_err(|_e| Error::Custom(format!("invalid method {}", method)))?;

            if missing_sign_flag(sig_header, key) {
                return Err(State::Custom(
                    "signing requires --sign <header-value> or --key <key-value>".to_string(),
                ));
            }

            let services = if !yaml_file.is_empty() {
                parse_yaml_file(yaml_file, regex, filter, envsubst).await?
            } else {
                Services::default()
            };

            let openfass_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or_default();
            let gateway_address = get_gateway_url(
                gateway,
                DEFAULT_GATEWAY,
                services.provider.gateway_url.as_str(),
                openfass_url.as_str(),
            );

            #[cfg(target_os = "unix")]
            {
                use std::os::unix::fs::PermissionsExt;
                use utility::MODE_CHAR_DEVICE;
                let stat = std::fs::metadata(std::io::stdin())
                    .map_err(|e| State::Custom(e.to_string()))?;
                if (stat.mode() & MODE_CHAR_DEVICE) != 0 {
                    std::io::stderr()
                        .write(b"Reading from STDIN - hit (Control + D) to stop.\n")
                        .map_err(|e| State::Custom(e.to_string()));
                }
            }
            let mut function_input = Vec::new();
            std::io::stdin()
                .read_to_end(&mut function_input)
                .map_err(|e| State::Custom(format!("unable to read standard input: {}", e)))?;

            let signed_header;
            if !sig_header.is_empty() {
                signed_header =
                    generate_signed_header(function_input.as_slice(), key.as_ref(), sig_header)?;
                //headers = append(headers, signedHeader)
                headers.push(signed_header.as_str());
            }

            let response = invoke_function(
                gateway_address.as_str(),
                function_name,
                &function_input,
                content_type,
                &query,
                &headers,
                invoke_async,
                method,
                tls_insecure,
                namespace,
            )
            .await?;

            if !response.is_empty() {
                std::io::stdout()
                    .write(response.as_ref())
                    .map_err(|e| State::Custom(e.to_string()))?;
            }

            Err(State::Matched)
        } else {
            //todo investigate the output
            Ok(())
        }
    }
}
fn missing_sign_flag(header: &str, key: &str) -> bool {
    (!header.is_empty() && key.is_empty()) || (header.is_empty() && !key.is_empty())
}

fn generate_signed_header(
    message: &[u8],
    key: &[u8],
    header_name: &str,
) -> utility::Result<String> {
    if header_name.is_empty() {
        Err(Error::Custom(
            "signed header must have a non-zero length".to_string(),
        ))
    } else {
        // Create HMAC-SHA256 instance which implements `Mac` trait
        let mut mac = HmacSha1::new_from_slice(key)
            .map_err(|_e| Error::Custom("HMAC can take key of any size".to_string()))?;
        mac.update(message);

        // `result` has type `Output` which is a thin wrapper around array of
        // bytes for providing constant time equality check
        let result = mac.finalize();
        // To get underlying array use `into_bytes` method, but be careful, since
        // incorrect use of the code value may permit timing attacks which defeat
        // the security provided by the `Output`
        let code_bytes = result.into_bytes();
        let hash;
        unsafe {
            let signature = std::str::from_utf8_unchecked(&code_bytes);
            hash = format!("{}={}={}", header_name, "sha1", signature);
        }

        Ok(hash)
    }

    // hash := hmac.Sign(message, []byte(key))
    // signature := hex.EncodeToString(hash)
    // signedHeader := fmt.Sprintf(`%s=%s=%s`, headerName, "sha1", string(signature[:]))
    //
    // return signedHeader, nil
}
