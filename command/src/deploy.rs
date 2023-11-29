use crate::error::check_tls_insecure;
use crate::faas::{check_and_set_default_yaml, DEFAULT_GATEWAY, DEFAULT_NETWORK};
use crate::priority::{get_gateway_url, get_namespace, OPENFAAS_URL_ENVIRONMENT};
use crate::validate::validate_language_flag;
use crate::{CommandAppend, State};
use builder::build::get_image_tag_values;
use clap::{App, Arg, ArgMatches, SubCommand};
use proxy::auth::ClientAuthE;
use proxy::client::Client;
use proxy::deploy::{DeployFunctionSpec, FunctionResourceRequest};
use reqwest::StatusCode;
use schema::image::{build_image_name, BuildFormat, TBuildFormat};
use stack::language_template::parse_yaml_for_language_template;
use stack::schema::{EnvironmentFile, Function};
use stack::stack::parse_yaml_file;
use std::collections::HashMap;
use std::io::ErrorKind;
use utility::{Error, Result};
//use crate::faas::check_and_set_default_yaml;

pub(crate) struct Deploy;

impl CommandAppend for Deploy {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("deploy")
                .about(r#"Deploys OpenFaaS function containers either via the supplied YAML config using
the "--yaml" flag (which may contain multiple function definitions), or directly
via flags. Note: --replace and --update are mutually exclusive.`,
	Example: `  faas-cli deploy -f https://domain/path/myfunctions.yml
  faas-cli deploy -f ./stack.yml
  faas-cli deploy -f ./stack.yml --label canary=true
  faas-cli deploy -f ./stack.yml --annotation user=true
  faas-cli deploy -f ./stack.yml --filter "*gif*" --secret dockerhuborg
  faas-cli deploy -f ./stack.yml --regex "fn[0-9]_.*"
  faas-cli deploy -f ./stack.yml --replace=false --update=true
  faas-cli deploy -f ./stack.yml --replace=true --update=false
  faas-cli deploy -f ./stack.yml --tag sha
  faas-cli deploy -f ./stack.yml --tag branch
  faas-cli deploy -f ./stack.yml --tag describe
  faas-cli deploy --image=alexellis/faas-url-ping --name=url-ping
  faas-cli deploy --image=my_image --name=my_fn --handler=/path/to/fn/
                  --gateway=http://remote-site.com:8080 --lang=python
                  --env=MYVAR=myval` "#)
                .args_from_usage(
                    " --fprocess [fprocess]          'fprocess value to be run as a serverless function by the watchdog'
                          --handler [handler]             'Directory with handler for function, e.g. handler.js'
                          --image   [image]                'Docker image name to build'
                          --lang    [lang]                  'Programming language template'
                          --name    [name]                  'Name of the deployed function'
                          --network [network]               'Name of the network'
                          -n ,--namespace [namespace]       'Namespace of the function'
                         --replace                          'Remove and re-create existing function(s)'
                         --update                           'Perform rolling update on existing function(s)'
                         --readonly                         'Force the root container filesystem to be read only'
                         --tag [tag]                         'Override latest tag on function Docker image, accepts latest, sha, branch, or describe'
                         --tls-no-verify                      'Disable TLS validation'
                         -k ,--token  [token]                     'Pass a JWT token to use instead of basic auth'
                         --read-template                       'Read the function's template'
            ")
                .arg(
                    Arg::with_name("env")
                        .help("Set one or more environment variables --env e1=v1 ")
                        .long("env")
                        .short("e")
                        .takes_value(true)
                        .global(true)
                        .multiple(true)
                ).arg(
                    Arg::with_name("label")
                        .help("Set one or more label (LABEL=VALUE) ")
                        .long("label")
                        //.short("l")
                        .takes_value(true)
                        .global(true)
                        .multiple(true)
                ).arg(
                Arg::with_name("annotation")
                    .help(" Set one or more annotation (ANNOTATION=VALUE)")
                    .long("annotation")
                    .takes_value(true)
                    .global(true)
                    .multiple(true)
            ).arg(
                Arg::with_name("constraint")
                    .help("Apply a constraint to the function")
                    .long("constraint")
                    .takes_value(true)
                    .global(true)
                    .multiple(true)
            ).arg(
                Arg::with_name("secret")
                    .help("Give the function access to a secure secret")
                    .long("secret")
                    .takes_value(true)
                    .global(true)
                    .multiple(true)
            )
        );
        //todo check bash-completion in clap
        //_ = deployCmd.Flags().SetAnnotation("handler", cobra.BashCompSubdirsInDir, []string{})

        app
    }
}

impl Deploy {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(dp_args) = args.subcommand_matches("deploy") {
            deploy_from_args(dp_args).await?;
            Err(State::Matched)
            //pushStack(&services, parallel, tagFormat)
        } else {
            Ok(())
        }
    }
}

pub(crate) async fn deploy_from_args(dp_args: &ArgMatches<'_>) -> Result<()> {
    let gateway_arg = dp_args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
    let regex = dp_args.value_of("regex").unwrap_or("");
    let filter = dp_args.value_of("filter").unwrap_or("");
    let envsubst = true; //dp_args.is_present("envsubst");
    let yaml_file = dp_args
        .value_of("yaml")
        .unwrap_or(check_and_set_default_yaml().unwrap_or_default());

    let update = true; //dp_args.is_present("update");
    let read_template = true; //dp_args.is_present("read-template");
    let replace = dp_args.is_present("replace");
    let read_only_root_filesystem = dp_args.is_present("readonly");
    let tls_insecure = dp_args.is_present("tls-no-verify");

    let language = dp_args.value_of("lang").unwrap_or_default();
    let network = dp_args.value_of("network").unwrap_or(DEFAULT_NETWORK);
    let image = dp_args.value_of("image").unwrap_or_default();
    let fprocess = dp_args.value_of("fprocess").unwrap_or_default();
    let function_name = dp_args.value_of("name").unwrap_or_default();

    let token = dp_args.value_of("token").unwrap_or_default();
    let function_namespace = dp_args.value_of("namespace").unwrap_or_default();
    let tag = dp_args.value_of("tag").unwrap_or("");

    let mut tag_format: Option<BuildFormat> = None;
    tag_format.set(Some(tag.to_string()))?;

    let secrets: Vec<&str> = dp_args.values_of("secret").unwrap_or_default().collect();
    let label_opts: Vec<String> = dp_args
        .values_of("label")
        .unwrap_or_default()
        .map(|m| m.to_string())
        .collect();
    let envvar_opts: Vec<String> = dp_args
        .values_of("env")
        .unwrap_or_default()
        .map(|m| m.to_string())
        .collect();
    let annotation_opts: Vec<String> = dp_args
        .values_of("annotation")
        .unwrap_or_default()
        .map(|m| m.to_string())
        .collect();

    let constraints: Vec<&str> = dp_args
        .values_of("constraint")
        .unwrap_or_default()
        .collect();

    let (language, _) = validate_language_flag(language);

    if update && replace {
        println!(
            r#"Cannot specify --update and --replace at the same time. One of --update or --replace must be false.
                    --replace    removes an existing deployment before re-creating it
                                --update     performs a rolling update to a new function image or configuration (default true)"#
        );
        return Err(Error::Custom(
            "cannot specify --update and --replace at the same time".to_string(),
        ));
    }
    let mut services = Default::default();

    if !yaml_file.is_empty() {
        let mut parsed_svc = parse_yaml_file(yaml_file, regex, filter, envsubst).await?;

        let openfass_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or_default();
        parsed_svc.provider.gateway_url = get_gateway_url(
            gateway_arg,
            DEFAULT_GATEWAY,
            parsed_svc.provider.gateway_url.as_str(),
            openfass_url.as_str(),
        );
        services = parsed_svc;
    }

    //transport := GetDefaultCLITransport(tls_Insecure, &commandTimeout)
    //ctx := context.Background()

    let mut failed_status_code: HashMap<String, u16> = HashMap::new();

    if !services.functions.is_empty() {
        let cli_auth = ClientAuthE::new(token, services.provider.gateway_url.as_str())?;

        let proxy_client = cli_auth.get_client(services.provider.gateway_url.as_str())?;
        // proxyClient, err := proxy.NewClient(cliAuth, services.Provider.GatewayURL, transport, &commandTimeout)

        for (k, mut function) in services.functions {
            let mut function_secrets = secrets.iter().map(|s| s.to_string()).collect();

            function.name = k.clone();
            println!("Deploying: {}.\n", function.name);
            let function_constraints: Vec<String> = if !&function.constraints.is_empty() {
                function.constraints.clone()
            } else if !&constraints.is_empty() {
                constraints.iter().map(|m| m.to_string()).collect()
            } else {
                Default::default()
            };

            if !function.secrets.is_empty() {
                function_secrets = merge_slice(function.secrets.clone(), function_secrets);
            }

            // Check if there is a functionNamespace flag passed, if so, override the namespace value
            // defined in the stack.yaml
            function.namespace = get_namespace(function_namespace, function.namespace.as_str());

            let file_environment = read_files(&function.environment_file)?;

            let label_map = function.labels.clone();
            let label_arg_map = parse_map(&label_opts, "label")
                .map_err(|e| Error::Custom(format!("error parsing labels: {}", e)))?;

            let all_labels = merge_map(label_map, label_arg_map);
            let all_env = compile_environment(
                &envvar_opts,
                function.environment.as_ref().unwrap_or(&HashMap::new()),
                &file_environment,
            )?;

            if read_template {
                // Get FProcess to use from the ./template/template.yml, if a template is being used

                if language_exists_not_dockerfile(function.language.as_str()) {
                    function.fprocess =derive_fprocess(&function).await
                        .map_err(|e| Error::Custom(format!(r#"template directory may be missing or invalid, please run "faas-cli template pull",
                                                  Error: {} "#, e)))?;
                }
            }

            let function_resource_request = FunctionResourceRequest {
                limits: Some(function.limits.clone()),
                requests: Some(function.requests.clone()),
            };

            let annotations = function.annotations.clone();
            let annotation_args = parse_map(&annotation_opts, "annotation")
                .map_err(|e| Error::Custom(format!("error parsing annotations: {}", e)))?;

            let all_annotations = merge_map(annotations, annotation_args);
            let (branch, sha) = get_image_tag_values(tag_format.unwrap_or_default())?;
            function.image = build_image_name(
                tag_format.unwrap_or_default(),
                function.image.as_str(),
                sha.as_str(),
                branch.as_str(),
            );

            if read_only_root_filesystem {
                function.readonly_root_filesystem = read_only_root_filesystem;
            }

            let spec = DeployFunctionSpec {
                fprocess: function.fprocess.unwrap_or_default().clone(),
                function_name: function.name.clone(),
                image: function.image.clone(),
                registry_auth: "".to_string(),
                language: function.language.clone(),
                replace,
                env_vars: all_env,
                network: "".to_string(),
                constraints: function_constraints,
                update,
                secrets: function_secrets,
                labels: all_labels,
                annotations: all_annotations,
                function_resource_request,
                read_only_root_filesystem: function.readonly_root_filesystem,
                tls_insecure,
                token: token.to_string(),
                namespace: function.namespace,
            };
            let msg = check_tls_insecure(services.provider.gateway_url.as_str(), spec.tls_insecure);
            if !msg.is_empty() {
                println!("{}", msg);
            }

            let (status_code, output) = proxy_client.deploy_function(&spec).await?;
            println!("{}", output);
            //here
            if bad_status_code(status_code) {
                failed_status_code.insert(k, status_code);
            }
        }
    } else {
        if image.is_empty() || function_name.is_empty() {
            return Err(Error::Custom(
                "To deploy a function give --yaml/-f or a --image and --name flag".to_string(),
            ));
        }

        let openfaas_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or_default();
        let gateway = get_gateway_url(
            gateway_arg,
            DEFAULT_GATEWAY,
            services.provider.gateway_url.as_str(),
            openfaas_url.as_str(),
        );
        let cli_auth = ClientAuthE::new(token, gateway.as_str())?;
        let proxy_client = cli_auth.get_client(gateway.as_str())?;
        // proxyClient, err := proxy.NewClient(cliAuth, gateway, transport, &commandTimeout)

        // default to a readable filesystem until we get more input about the expected behavior
        // and if we want to add another flag for this case
        let default_read_only_rfs = false;
        let deploy_flags = DeployFlags {
            envvar_opts: &envvar_opts,
            replace,
            update,
            read_only_root_filesystem,
            constraints: constraints.iter().map(|s| s.to_string()).collect(),
            secrets: secrets.iter().map(|s| s.to_string()).collect(),
            label_opts: &label_opts,
            annotation_opts: &annotation_opts,
        };
        let status_code = deploy_image(
            &proxy_client,
            image.to_string(),
            fprocess.to_string(),
            function_name.to_string(),
            "".to_string(),
            deploy_flags,
            tls_insecure,
            default_read_only_rfs,
            token.to_string(),
            function_namespace.to_string(),
            language.to_string(),
            network.to_string(),
            gateway_arg,
        )
        .await?;

        if status_code == StatusCode::BAD_REQUEST.as_u16() {
            failed_status_code.insert(function_name.to_string(), status_code);
        }
    }

    deploy_failed(&failed_status_code)?;
    Ok(())
}

pub(crate) fn deploy_failed(status: &HashMap<String, u16>) -> Result<()> {
    if status.is_empty() {
        Ok(())
    } else {
        let mut all_errors: Vec<String> = Vec::new();
        for (name, status) in status {
            all_errors.push(format!(
                "Function '{}' failed to deploy with status code: {}",
                name, status
            ));
        }
        Err(Error::Custom(all_errors.join("\n")))
    }
}
pub(crate) struct DeployFlags<'s> {
    pub(crate) envvar_opts: &'s Vec<String>,
    pub(crate) replace: bool,
    pub(crate) update: bool,
    pub(crate) read_only_root_filesystem: bool,
    pub(crate) constraints: Vec<String>,
    pub(crate) secrets: Vec<String>,
    pub(crate) label_opts: &'s Vec<String>,
    pub(crate) annotation_opts: &'s Vec<String>,
}

/// deployImage deploys a function with the given image
pub(crate) async fn deploy_image(
    client: &Client<'_>,
    image: String,
    fprocess: String,
    function_name: String,
    registry_auth: String,
    deploy_flags: DeployFlags<'_>,
    tls_insecure: bool,
    read_only_root_filesystem: bool,
    token: String,
    namespace: String,
    language: String,
    network: String,
    gateway: &str,
) -> Result<u16> {
    let read_only_rfs = deploy_flags.read_only_root_filesystem || read_only_root_filesystem;
    let envvars = parse_map(deploy_flags.envvar_opts, "env")?;

    let label_map = parse_map(deploy_flags.label_opts, "label")?;

    let annotation_map = parse_map(deploy_flags.annotation_opts, "annotation")?;

    let spec = DeployFunctionSpec {
        fprocess,
        function_name,
        image,
        registry_auth,
        language,
        replace: deploy_flags.replace,
        env_vars: envvars,
        network,
        constraints: deploy_flags.constraints,
        update: deploy_flags.update,
        secrets: deploy_flags.secrets,
        labels: label_map,
        annotations: annotation_map,
        function_resource_request: Default::default(),
        read_only_root_filesystem: read_only_rfs,
        tls_insecure,
        token,
        namespace,
    };

    let msg = check_tls_insecure(gateway, spec.tls_insecure);
    if !msg.is_empty() {
        println!("{}", msg);
    }
    let (status_code, output) = client.deploy_function(&spec).await?;
    println!("{}", output);
    Ok(status_code)
}

pub(crate) fn merge_slice(values: Vec<String>, overlay: Vec<String>) -> Vec<String> {
    let mut results = Vec::new();
    let mut add: HashMap<String, bool> = HashMap::new();

    for value in overlay {
        results.push(value.clone());
        add.insert(value, true);
    }

    for value in values {
        if add.get(value.as_str()).is_none() {
            results.push(value);
        }
    }
    results
}

pub(crate) fn parse_map(envvars: &Vec<String>, key_name: &str) -> Result<HashMap<String, String>> {
    let mut result: HashMap<String, String> = HashMap::new();
    for envvar in envvars {
        let trimed = envvar.replace(' ', "");
        let s: Vec<&str> = trimed.splitn(2, '=').collect();
        if s.len() != 2 {
            return Err(Error::Custom(
                "label format is not correct, needs key=value".to_string(),
            ));
        }

        let envvar_name = s[0];
        let envvar_value = s[1];
        if envvar_name.is_empty() {
            return Err(Error::Custom(format!(
                "empty {} name: [{}]",
                key_name, envvar
            )));
        }
        if envvar_name.is_empty() {
            return Err(Error::Custom(format!(
                "empty {} name: [{}]",
                key_name, envvar
            )));
        }

        result.insert(envvar_name.to_string(), envvar_value.to_string());
    }
    Ok(result)
}

pub(crate) fn read_files(files: &Vec<String>) -> Result<HashMap<String, String>> {
    let mut envs = HashMap::new();

    for file in files {
        let bytes_out = std::fs::read_to_string(file)?;
        let env_file: EnvironmentFile =
            serde_json::from_str(bytes_out.as_str()).map_err(|e| Error::Custom(e.to_string()))?;

        for (k, v) in env_file.environment {
            envs.insert(k, v);
        }
    }

    Ok(envs)
}

pub(crate) fn merge_map(
    i: HashMap<String, String>,
    j: HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged: HashMap<String, String> = HashMap::new();

    for (k, v) in i {
        merged.insert(k, v);
    }
    for (k, v) in j {
        merged.insert(k, v);
    }
    merged
}

pub(crate) fn compile_environment(
    envvar_opts: &Vec<String>,
    yaml_environment: &HashMap<String, String>,
    file_environment: &HashMap<String, String>,
) -> Result<HashMap<String, String>> {
    let envvar_argument = parse_map(envvar_opts, "env")
        .map_err(|err| Error::Custom(format!("error parsing envvars: {}", err)))?;

    let function_and_stack = merge_map(yaml_environment.to_owned(), file_environment.to_owned());
    Ok(merge_map(function_and_stack, envvar_argument))
}

fn language_exists_not_dockerfile(language: &str) -> bool {
    !language.is_empty() && language.to_ascii_uppercase() != "dockerfile"
}

async fn derive_fprocess(function: &Function) -> Result<Option<String>> {
    let path_to_temp_yaml = format!("./template/{}/template.yml", function.language);
    if let Err(e) = std::fs::metadata(&path_to_temp_yaml) {
        if e.kind() == ErrorKind::NotFound {
            return Err(Error::Io(e));
        }
    }
    let parsed_lang_temp = parse_yaml_for_language_template(path_to_temp_yaml.as_str()).await?;

    Ok(parsed_lang_temp.fprocess)
}

pub(crate) fn bad_status_code(status: u16) -> bool {
    status != StatusCode::ACCEPTED.as_u16() && status != StatusCode::OK.as_u16()
}
