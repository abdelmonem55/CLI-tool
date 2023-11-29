use crate::error::check_tls_insecure;
use crate::faas::DEFAULT_GATEWAY;
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::{State, SubCommandAppend};
use clap::{App, ArgMatches, SubCommand};
use proxy::auth::ClientAuthE;
use utility::faas::types::model::Secret;

pub(crate) struct SecretList;

impl SubCommandAppend for SecretList {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app =
            SubCommand::with_name("list")
                .alias("ls")
                .about(r#"faas-cli store list
  faas-cli store list --verbose
  faas-cli store list --url https://host:port/store.json`"#)
                .args_from_usage("--tls-no-verify     'Disable TLS validation'
                   -k, --token [token]                      'Pass a JWT token to use instead of basic auth'
                   -n, --namespace  [namespace]             'Namespace of the function'
                ");

        app
    }
}

impl SecretList {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(l_args) = args.subcommand_matches("list") {
            let gateway = l_args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
            let token = l_args.value_of("token").unwrap_or_default();
            let namespace = l_args.value_of("namespace").unwrap_or_default();
            let tls_no_verify = l_args.is_present("tls-no-verify");

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
            let secrets = client.get_secret_list(namespace).await?;

            if secrets.is_empty() {
                colour::yellow!("No secrets found.\n")
            } else {
                colour::green!("{}", render_secret_list(secrets));
            }

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

fn render_secret_list(secrets: Vec<Secret>) -> String {
    let mut fmt = "NAME\n".to_string();
    for secret in secrets {
        fmt.push_str(secret.name.as_str());
        fmt.push('\n');
    }
    fmt
}
