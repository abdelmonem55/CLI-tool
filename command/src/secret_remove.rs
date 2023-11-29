use crate::error::check_tls_insecure;
use crate::faas::DEFAULT_GATEWAY;
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::{State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};
use proxy::auth::ClientAuthE;
use utility::faas::types::model::Secret;

pub(crate) struct SecretRemove;

impl SubCommandAppend for SecretRemove {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app =
            SubCommand::with_name("remove")
                .alias("rm")
                .about(r#"faas-cli secret remove NAME
faas-cli secret remove NAME --gateway=http://127.0.0.1:8080"#)
                .arg(Arg::with_name("SECRET-NAME")
                    .index(1)
                    .help("secret name")
                    .required(true)
                )
                //.arg(Arg::with_name("STDIN")
                //     .index(2)
                //     .help("secret from stdin")
                // )
                .args_from_usage("
                   --tls-no-verify                      'Disable TLS validation'
                   -k, --token [token]                      'Pass a JWT token to use instead of basic auth'
                   -n, --namespace  [namespace]             'Namespace of the function'
                ");

        app
    }
}

impl SecretRemove {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(r_args) = args.subcommand_matches("remove") {
            let secret_name = r_args
                .value_of("SECRET-NAME")
                .ok_or(State::Custom("secret name is required".to_string()))?;
            let function_namespace = r_args.value_of("namespace").unwrap_or_default();
            let gateway = args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
            let token = r_args.value_of("token").unwrap_or_default();
            let tls_no_verify = r_args.is_present("tls-no-verify");

            let secret = Secret {
                name: secret_name.to_string(),
                namespace: function_namespace.to_string(),
                value: "".to_string(),
            };

            let openfaas_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or_default();
            let gateway_address =
                get_gateway_url(gateway, DEFAULT_GATEWAY, "", openfaas_url.as_str());

            let msg = check_tls_insecure(gateway_address.as_str(), tls_no_verify);
            if !msg.is_empty() {
                colour::yellow!("{}\n", msg);
            }
            let client_auth = ClientAuthE::new(token, gateway_address.as_str())?;
            //transport := GetDefaultCLITransport(tlsInsecure, &commandTimeout)
            let client = client_auth.get_client(gateway_address.as_str())?;
            client.remove_secret(&secret).await?;
            colour::green!("removed {}");
            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}
