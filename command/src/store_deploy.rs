use crate::deploy::{bad_status_code, deploy_failed, deploy_image, DeployFlags};
use crate::faas::DEFAULT_GATEWAY;
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::store::{
    filter_store_list, get_target_platform, store_find_function, store_list, DEFAULT_STORE,
};
use crate::{State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};
use proxy::auth::ClientAuthE;
use std::collections::HashMap;

pub(crate) struct StoreDeploy;

impl SubCommandAppend for StoreDeploy {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app =
            SubCommand::with_name("deploy")
                .about(r#"Same as faas-cli deploy except that function is pre-loaded with arguments from the store`,
	Example: `  faas-cli store deploy figlet
  faas-cli store deploy figlet \
    --gateway=http://127.0.0.1:8080 \
    --env=MYVAR=myval"#)
                .arg(
                    Arg::with_name("FUNCTION-NAME")
                        .help("function name")
                      //  .global(true)
                        .index(1)
                        .required(true)
                )
                .arg(
                    Arg::with_name("env")
                        .help("Adds one or more environment variables to the defined ones by store (ENVVAR=VALUE)")
                        .long("env")
                        .short("e")
                        .takes_value(true)
                        .global(true)
                        .multiple(true)
                ).arg(
                Arg::with_name("label")
                    .help("Add a label for Docker image (LABEL=VALUE)")
                    .long("label")
                    //.short("l")
                    .takes_value(true)
                    .global(true)
                    .multiple(true)
            ).arg(
                    Arg::with_name("secure")
                        .help("Give the function access to a secure secret")
                        .long("build-label")
                        .takes_value(true)
                        .global(true)
                        .multiple(true)
                )
                .arg(
                    Arg::with_name("constraint")
                        .help("Apply a constraint to the function")
                        .long("constraint")
                        .takes_value(true)
                        .global(true)
                        .multiple(true)
                )
                .arg(
                    Arg::with_name("annotation")
                        .help("Set one or more annotation (ANNOTATION=VALUE)")
                        .long("annotation")
                        .takes_value(true)
                        .global(true)
                        .multiple(true)
                )
                .args_from_usage("
                    --network       [network]           'Name of the network'
                    --name          [name]              'Name of the deployed function (overriding name from the store)'
                    -n ,--namespace [namespace]         'Namespace of the function'
                    --replace                           'Replace any existing function'
                    --update                            'Update existing functions'
                   --tls-no-verify                      'Disable TLS validation'
                   -k ,--token [token]                  'Pass a JWT token to use instead of basic auth'
                   ");
        app
    }
}

impl StoreDeploy {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(d_args) = args.subcommand_matches("deploy") {
            let store_address = args.value_of("url").unwrap_or(DEFAULT_STORE);
            let gateway_arg = "http://107.21.148.190:31112"; //args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);

            let platform_value = d_args.value_of("platform").unwrap_or_default();
            let requested_store_fn = d_args.value_of("FUNCTION-NAME").ok_or(State::Custom(
                "function name must be set at index 0 like 'faas store deploy NAME'".to_string(),
            ))?;
            let function_name = d_args.value_of("name").unwrap_or_default();
            let token = d_args.value_of("token").unwrap_or_default();
            let namespace = d_args.value_of("namespace").unwrap_or_default();

            let replace = d_args.is_present("replace");
            let update = true; //d_args.is_present("replace");
            let read_only_root_filesystem = args.is_present("readonly");
            let tls_insecure = d_args.is_present("tls-no-verify");

            let mut env_var_opts: Vec<String> = d_args
                .values_of("env")
                .unwrap_or_default()
                .map(|en| en.to_string())
                .collect();
            let mut label_opts: Vec<String> = d_args
                .values_of("label")
                .unwrap_or_default()
                .map(|en| en.to_string())
                .collect();
            let mut annotation_opts: Vec<String> = d_args
                .values_of("annotation")
                .unwrap_or_default()
                .map(|en| en.to_string())
                .collect();
            let secrets: Vec<String> = d_args
                .values_of("secret")
                .unwrap_or_default()
                .map(|en| en.to_string())
                .collect();
            let constraints: Vec<String> = d_args
                .values_of("constraint")
                .unwrap_or_default()
                .map(|en| en.to_string())
                .collect();

            // if len(args) < 1 {
            //     return fmt.Errorf("please provide the function name")
            // }
            let target_platform = get_target_platform(platform_value);
            let store_items = store_list(store_address).await?;
            let platform_functions = filter_store_list(store_items, target_platform.as_str());
            let item = store_find_function(requested_store_fn, &platform_functions).ok_or(
                State::Custom(format!(
                    "function '{}' not found for platform '{}'",
                    requested_store_fn, target_platform
                )),
            )?;

            // Add the store environment variables to the provided ones from cmd
            for (k, v) in &item.environment {
                let env = format!("{}={}", k, v);
                env_var_opts.push(env);
            }

            // Add the store labels to the provided ones from cmd
            for (k, v) in &item.labels {
                let label = format!("{}={}", k, v);
                label_opts.push(label);
            }

            for (k, v) in &item.annotations {
                let annotation = format!("{}={}", k, v);
                annotation_opts.push(annotation);
            }

            // Use the network from manifest if not changed by user
            let network = d_args.value_of("network").unwrap_or(item.network.as_str());

            let item_name = if !function_name.is_empty() {
                function_name
            } else {
                item.name.as_str()
            };
            let image_name = item
                .get_image_name(target_platform.as_str())
                .map(|s| s.to_owned())
                .unwrap_or_default();

            let openfaas_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or_default();
            let gateway = get_gateway_url(gateway_arg, DEFAULT_GATEWAY, "", openfaas_url.as_str());
            let cli_auth = ClientAuthE::new(token, gateway.as_str())?;
            let proxy_client = cli_auth.get_client(gateway.as_str())?;

            let deploy_flags = DeployFlags {
                envvar_opts: &env_var_opts,
                replace,
                update,
                read_only_root_filesystem,
                constraints,
                secrets,
                label_opts: &label_opts,
                annotation_opts: &annotation_opts,
            };
            //transport := GetDefaultCLITransport(tlsInsecure, &commandTimeout)
            //proxyClient, err := proxy.NewClient(cliAuth, gateway, transport, &commandTimeout)
            let status_code = deploy_image(
                &proxy_client,
                image_name,
                item.fprocess.clone(),
                item_name.to_string(),
                "".to_string(),
                deploy_flags,
                tls_insecure,
                read_only_root_filesystem,
                token.to_string(),
                namespace.to_string(),
                "".to_string(),
                network.to_string(),
                gateway_arg,
            )
            .await?;

            if bad_status_code(status_code) {
                let mut failed_code = HashMap::new();
                failed_code.insert(item_name.to_string(), status_code);
                deploy_failed(&failed_code)?;
            }

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}
