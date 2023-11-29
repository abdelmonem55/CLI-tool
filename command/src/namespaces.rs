use crate::faas::DEFAULT_GATEWAY;
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::{CommandAppend, State};
use clap::{App, ArgMatches, SubCommand};
use proxy::auth::ClientAuthE;

pub(crate) struct Namespaces;

impl CommandAppend for Namespaces {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("namespaces")
                .about(
                    r#"Lists OpenFaaS namespaces either on a local or remote gateway`,
	Example: `  faas-cli namespaces
  faas-cli namespaces --gateway https://127.0.0.1:8080`,"#,
                )
                .args_from_usage(
                    "-k ,--token [token] 'Pass a JWT token to use instead of basic auth'
            --tls-no-verify 'Disable TLS validation'
            ",
                ),
        );
        app
    }
}

impl Namespaces {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(ns_args) = args.subcommand_matches("namespaces") {
            let gateway = args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
            let token = ns_args.value_of("token").unwrap_or("");

            let openfass_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or_default();
            let gateway_address =
                get_gateway_url(gateway, DEFAULT_GATEWAY, "", openfass_url.as_str());

            let cli_auth = ClientAuthE::new(token, gateway_address.as_str())?;

            //transport := GetDefaultCLITransport(tlsInsecure, &commandTimeout)
            //client, err := proxy.NewClient(cliAuth, gatewayAddress, transport, &commandTimeout)
            let mut client = cli_auth.get_client(gateway_address.as_str())?;
            let namespaces = client.list_namesapces().await?;
            // namespaces, err := client.ListNamespaces(context.Background())
            print_namespaces(&namespaces);

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

fn print_namespaces(namespaces: &Vec<String>) {
    println!("Namespaces:\n");
    for ns in namespaces {
        println!(" - {}", ns);
    }
}
