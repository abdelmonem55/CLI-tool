use crate::faas::{check_and_set_default_yaml, DEFAULT_GATEWAY};
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::{CommandAppend, State};
use clap::{App, ArgMatches, SubCommand};
use proxy::auth::ClientAuthE;
use schema::describe::FunctionDescription;
use stack::stack::parse_yaml_file;

pub(crate) struct Describe;

impl CommandAppend for Describe {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand( SubCommand::with_name("describe")
            .aliases(&["rm","delete"])
            .about(r#"Display details of an OpenFaaS function`,
	Example: `faas-cli describe figlet
faas-cli describe env --gateway http://127.0.0.1:8080
faas-cli describe echo -g http://127.0.0.1.8080`"#)
            .args_from_usage("<FUNCTION-NAME>
                   --tls-no-verify     'Disable TLS validation'
                   -k, --token [token]                      'Pass a JWT token to use instead of basic auth'
                   -n, --namespace  [namespace]             'Namespace of the function'
                ")
        );

        app
    }
}

impl Describe {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(r_args) = args.subcommand_matches("describe") {
            let gateway = r_args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
            let token = r_args.value_of("token").unwrap_or_default();
            let function_namespace = r_args.value_of("namespace").unwrap_or_default();
            //let tls_no_verify = r_args.is_present("tls-no-verify");
            let envsubst = true;

            let function_name = r_args.value_of("FUNCTION-NAME").ok_or(State::Custom(
                "function name is required like: faas-cli describe FUNCTION_NAME".to_string(),
            ))?;

            let yaml_file = r_args
                .value_of("yaml")
                .unwrap_or(check_and_set_default_yaml().unwrap_or_default());

            // var services stack.Services
            // var gatewayAddress string
            // var yamlGateway string

            let yaml_gateway = if !yaml_file.is_empty() {
                let svcs = parse_yaml_file(yaml_file, "", "", envsubst).await?;
                svcs.provider.gateway_url

                // if parsedServices != nil {
                //     services = *parsedServices
                //     yamlGateway = services.Provider.GatewayURL
                // }
            } else {
                String::new()
            };
            let openfaas_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or_default();
            let gateway_address = get_gateway_url(
                gateway,
                DEFAULT_GATEWAY,
                yaml_gateway.as_str(),
                openfaas_url.as_str(),
            );
            let client_auth = ClientAuthE::new(token, gateway_address.as_str())?;
            let mut client = client_auth.get_client(gateway_address.as_str())?;
            // transport := GetDefaultCLITransport(tlsInsecure, &commandTimeout)
            // proxyclient, err := proxy.NewClient(cliAuth, gatewayAddress, transport, &commandTimeout)
            let function = client
                .get_function_info(function_name, function_namespace)
                .await?;

            //To get correct value for invocation count from /system/functions endpoint
            let function_list = client.list_functions(function_namespace).await?;

            let mut invocation_count: isize = 0;
            for func in function_list {
                if func.name == function_name {
                    invocation_count = func.invocation_count as isize;
                    break;
                }
            }

            let status = if function.available_replicas > 0 {
                "Ready"
            } else {
                "Not Ready"
            };

            let (url, async_url) =
                get_function_urls(gateway_address.as_str(), function_name, function_namespace);

            let func_desc = FunctionDescription {
                name: function_name,
                status,
                replicas: function.replicas as i32,
                available_replicas: function.available_replicas as i32,
                invocation_count: invocation_count as i32,
                image: function.image.as_str(),
                env_process: function.env_process.as_str(),
                url: url.as_str(),
                async_url: async_url.as_str(),
                labels: &function.labels,
                annotations: &function.annotations,
            };

            print_function_description(&func_desc);

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

fn get_function_urls(
    gateway: &str,
    function_name: &str,
    function_namespace: &str,
) -> (String, String) {
    let gateway = gateway.trim_end_matches('/');

    let mut url = gateway.to_string() + "/function/" + function_name;
    let mut async_url = gateway.to_string() + "/async-function/" + function_name;

    if !function_namespace.is_empty() {
        url = url + "." + function_namespace;
        async_url = async_url + "." + function_namespace;
    }

    (url, async_url)
}

fn print_function_description(func_desc: &FunctionDescription) {
    let mut fmt = format!("{:width$}", "\nName:", width = 30) + func_desc.name;
    let str = format!("{:width$}", "\nStatus:", width = 30) + func_desc.status;
    fmt.push_str(str.as_str());

    let str =
        format!("{:width$}", "\nReplicas:", width = 30) + func_desc.replicas.to_string().as_str();
    fmt.push_str(str.as_str());

    let str = format!("{:width$}", "\nAvailable replicas:", width = 30)
        + func_desc.available_replicas.to_string().as_str();
    fmt.push_str(str.as_str());

    let str = format!("{:width$}", "\nInvocations:", width = 30)
        + func_desc.invocation_count.to_string().as_str();
    fmt.push_str(str.as_str());

    let str = format!("{:width$}", "\nImage:", width = 30) + func_desc.image;
    fmt.push_str(str.as_str());

    let str = format!("{:width$}", "\nFunction process:", width = 30) + func_desc.env_process;
    fmt.push_str(str.as_str());

    let str = format!("{:width$}", "\nURL:", width = 30) + func_desc.url;
    fmt.push_str(str.as_str());

    let str = format!("{:width$}", "\nAsync URL:", width = 30) + func_desc.async_url;
    fmt.push_str(str.as_str());

    if !func_desc.labels.is_empty() {
        // let str = format!("{:width$}", "\nLabels:", width = 30);
        fmt.push_str("\nLabels");

        for (key, value) in func_desc.labels {
            let str = format!("{:width$}", "\n", width = 30);
            fmt = fmt + str.as_str() + key.as_str() + " : " + value.as_str();
        }
    }

    if !func_desc.annotations.is_empty() {
        //let str = format!("{:width$}", "\nAnnotations:", width = 30);
        fmt.push_str("\nAnnotations");

        for (key, value) in func_desc.annotations {
            let str = format!("{:width$}", "\n", width = 30);
            fmt = fmt + str.as_str() + key.as_str() + " : " + value.as_str();
        }
    }
    colour::green!("{}", fmt);
}
