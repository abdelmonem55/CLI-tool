use crate::faas::{check_and_set_default_yaml, DEFAULT_GATEWAY};
use crate::priority::{get_gateway_url, get_namespace, OPENFAAS_URL_ENVIRONMENT};
use crate::{CommandAppend, State};
use clap::{App, ArgMatches, SubCommand};
use proxy::auth::ClientAuthE;
use stack::stack::parse_yaml_file;

pub(crate) struct Remove;

impl CommandAppend for Remove {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand( SubCommand::with_name("remove")
                .aliases(&["rm","delete"])
                .about(r#"Removes/deletes deployed OpenFaaS functions either via the supplied YAML config
using the "--yaml" flag (which may contain multiple function definitions), or by
explicitly specifying a function name.`,
	Example: `  faas-cli remove -f https://domain/path/myfunctions.yml
  faas-cli remove -f ./stack.yml
  faas-cli remove -f ./stack.yml --filter "*gif*"
  faas-cli remove -f ./stack.yml --regex "fn[0-9]_.*"
  faas-cli remove url-ping
  faas-cli remove img2ansi --gateway=http://remote-site.com:8080`"#)
                .args_from_usage("[FUNCTION-NAME]
                   --tls-no-verify     'Disable TLS validation'
                   -k, --token [token]                      'Pass a JWT token to use instead of basic auth'
                   -n, --namespace  [namespace]             'Namespace of the function'
                ")
        );

        app
    }
}

impl Remove {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(r_args) = args.subcommand_matches("remove") {
            let gateway = r_args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
            let token = r_args.value_of("token").unwrap_or_default();
            let function_namespace = r_args.value_of("namespace").unwrap_or_default();
            //let tls_no_verify = r_args.is_present("tls-no-verify");
            let envsubst = true;

            let function_name = r_args.value_of("FUNCTION-NAME").unwrap_or_default();

            let yaml_file = r_args
                .value_of("yaml")
                .unwrap_or(check_and_set_default_yaml().unwrap_or_default());

            // var services stack.Services
            // var gatewayAddress string
            // var yamlGateway string

            let (services, yaml_gateway) = if !yaml_file.is_empty() && function_name.is_empty() {
                let svcs = parse_yaml_file(yaml_file, "", "", envsubst).await?;
                let yaml_gateway = svcs.provider.gateway_url.clone();
                (Some(svcs), yaml_gateway)

                // if parsedServices != nil {
                //     services = *parsedServices
                //     yamlGateway = services.Provider.GatewayURL
                // }
            } else {
                (None, String::new())
            };
            let openfaas_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or_default();
            let gateway_address = get_gateway_url(
                gateway,
                DEFAULT_GATEWAY,
                yaml_gateway.as_str(),
                openfaas_url.as_str(),
            );

            let client_auth = ClientAuthE::new(token, gateway_address.as_str())?;
            let client = client_auth.get_client(gateway_address.as_str())?;
            // transport := GetDefaultCLITransport(tlsInsecure, &commandTimeout)
            // proxyclient, err := proxy.NewClient(cliAuth, gatewayAddress, transport, &commandTimeout)

            if services.is_some() && !services.as_ref().unwrap().functions.is_empty() {
                let services = services.unwrap();

                for (k, mut function) in services.functions {
                    function.namespace =
                        get_namespace(function_namespace, function.namespace.as_str());
                    function.name = k;
                    colour::green!("Deleting: {}.{}\n", function.name, function.namespace);
                    client
                        .delete_function(function.name.as_str(), function.namespace.as_str())
                        .await?;
                }
            } else {
                if function_name.is_empty() {
                    return Err(State::Custom(
                        "please provide the name of a function to delete".to_string(),
                    ));
                }
                colour::green!("Deleting: {}.{}\n", function_name, function_namespace);
                client
                    .delete_function(function_name, function_namespace)
                    .await?;
            }

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}
