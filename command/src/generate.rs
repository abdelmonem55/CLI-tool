use crate::deploy::{compile_environment, merge_map, parse_map, read_files};
use crate::faas::check_and_set_default_yaml;
use crate::store::DEFAULT_STORE;
use crate::{CommandAppend, State};
use builder::build::get_image_tag_values;
use clap::{App, Arg, ArgMatches, SubCommand};
use proxy::function_store::function_store_list;
use schema::image::{build_image_name, BuildFormat, TBuildFormat};
use schema::knative::v1::serving::{
    EnvPair, Secret, ServingServiceCRD, ServingSpecContainersContainerSpec, Volume, VolumeMount,
    API_VERSION_LATEST,
};
use schema::metadata::Metadata;
use schema::openfaas::v1::crd::{Spec, CRD};
use schema::store::v2::store::StoreFunction;
use stack::schema::{Function, Provider, Services};
use stack::stack::parse_yaml_file;
use std::collections::HashMap;
use utility::{Error, Result};

pub(crate) const DEFAULT_FUNCTION_NAMESPACE: &str = "";
const RESOURCE_KIND: &str = "Function";
const DEFAULT_API_VERSION: &str = "openfaas.com/v1";

pub(crate) struct Generate;

impl CommandAppend for Generate {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("generate")
                .about(r#"`The generate command creates kubernetes CRD YAML file for functions`,
	Example: `faas-cli generate --api=openfaas.com/v1 --yaml stack.yml | kubectl apply  -f -
faas-cli generate --api=openfaas.com/v1 -f stack.yml
faas-cli generate --api=serving.knative.dev/v1 -f stack.yml
faas-cli generate --api=openfaas.com/v1 --namespace openfaas-fn -f stack.yml
faas-cli generate --api=openfaas.com/v1 -f stack.yml --tag branch -n openfaas-fn`,"#)
                .args_from_usage("
                --from-store [from-store]       'generate using a store image'
                --api [api]                     'CRD API version e.g openfaas.com/v1, serving.knative.dev/v1'
                -n ,--namespace [namespace]     'Kubernetes namespace for functions'
                --tag  [tag]                    'Override latest tag on function Docker image, accepts latest, sha, branch, describe'
                --arch  [arch]                  'Desired image arch. (Default x86_64)'
                -u ,--url   [url]               'Alternative Store URL starting with http(s)://'
            ")
                .arg(
                    Arg::with_name("annotation")
                        .help("Any annotations you want to add (to store functions only) used like --annotation a1=str1 ")
                        .long("annotation")
                        .takes_value(true)
                        .global(true)
                        .multiple(true)
                )
        );
        app
    }
}

impl Generate {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(g_args) = args.subcommand_matches("generate") {
            // let gateway = args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
            let regex = args.value_of("regex").unwrap_or("");
            let filter = args.value_of("filter").unwrap_or("");
            let yaml_file = args
                .value_of("yaml")
                .unwrap_or(check_and_set_default_yaml().unwrap_or_default());

            let envsubst = true; //args.is_present("envsubst");

            let tag = g_args.value_of("tag").unwrap_or("");
            let from_store = g_args.value_of("from-store").unwrap_or("");
            let store_address = g_args.value_of("url").unwrap_or(DEFAULT_STORE);
            let function_namespace = g_args
                .value_of("namespace")
                .unwrap_or(DEFAULT_FUNCTION_NAMESPACE);

            let desired_arch = g_args.value_of("arch").unwrap_or("x86_64");
            let api = g_args.value_of("api").unwrap_or(DEFAULT_API_VERSION);
            let annotation_args: Vec<String> = g_args
                .values_of("annotation")
                .unwrap_or_default()
                .map(|s| s.to_string())
                .collect();

            let mut tag_format: Option<BuildFormat> = None;
            tag_format.set(Some(tag.to_string()))?;

            //var services stack.Services
            let annotations = parse_map(&annotation_args, "annotation")
                .map_err(|e| State::Custom(format!("error parsing annotations: {}", e)))?;
            let mut services = Default::default();
            if !from_store.is_empty() {
                services = Services {
                    version: "1.0".to_string(),
                    provider: Provider {
                        name: "openfaas".to_string(),
                        ..Default::default()
                    },
                    ..Default::default()
                };

                let items = function_store_list(store_address).await.map_err(|_| {
                    State::Custom(format!(
                        "Unable to retrieve functions from URL %s {}",
                        store_address
                    ))
                })?;

                let item = filter_store_item(&items, from_store)?;

                if item.images.get(desired_arch).is_none() {
                    let mut keys: Vec<String> = Vec::new();
                    for (k, _) in &item.images {
                        keys.push(k.to_owned());
                        return Err(State::Custom(format!(
                            "image for {} not found in store. \noptions: {:?}",
                            desired_arch, keys
                        )));
                    }

                    let all_annotations = merge_map(item.annotations.clone(), annotations);

                    services.functions.insert(
                        item.name.clone(),
                        Function {
                            name: item.name.clone(),
                            image: item
                                .images
                                .get(desired_arch)
                                .map(|i| i.to_owned())
                                .unwrap_or_default(),
                            labels: item.labels.clone(),
                            annotations: all_annotations,
                            environment: Some(item.environment.clone()),
                            fprocess: Some(item.fprocess.clone()),
                            ..Default::default()
                        },
                    );
                }
            } else if !yaml_file.is_empty() {
                let parsed_services = parse_yaml_file(yaml_file, regex, filter, envsubst).await;
                services = parsed_services?;
            }

            let (branch, version) = get_image_tag_values(tag_format.unwrap())?;
            //println!("{:?}\n{:?},{},{},{} {}",services,tag_format,function_namespace,branch,version,api);

            let objects_string = generate_crd_yaml(
                &services,
                tag_format.unwrap_or_default(),
                api,
                function_namespace,
                branch.as_str(),
                version.as_str(),
            )?;

            if !objects_string.is_empty() {
                println!("{}", objects_string);
            }
            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

fn filter_store_item(items: &Vec<StoreFunction>, from_store: &str) -> Result<StoreFunction> {
    let mut item = None;
    for val in items {
        if val.name == from_store {
            item = Some(val.clone());
            break;
        }
    }

    Ok(item.ok_or(Error::Custom(format!(
        "unable to find '{}' in store",
        from_store
    )))?)
}

///generateCRDYAML generates CRD YAML for functions
fn generate_crd_yaml(
    services: &Services,
    format: BuildFormat,
    api_version: &str,
    namespace: &str,
    branch: &str,
    version: &str,
) -> Result<String> {
    let mut objects_string = String::new();

    if !services.functions.is_empty() {
        if api_version == API_VERSION_LATEST {
            return generate_knative_v1_serving_service_crd_yaml(
                services,
                format,
                api_version,
                namespace,
                branch,
                version,
            );
        }

        let ordered_names = generate_function_order(&services.functions);

        for name in ordered_names {
            let function = services
                .functions
                .get(name.as_str())
                .unwrap_or(&Default::default())
                .to_owned();

            //read environment variables from the file
            let file_environment = read_files(&function.environment_file)?;

            // combine all environment variables
            let all_environment = compile_environment(
                &Vec::new(),
                &function.environment.unwrap_or_default(),
                &file_environment,
            )?;
            let metadate = Metadata {
                name: name.clone(),
                namespace: namespace.to_string(),
                ..Default::default()
            };
            let image_name = build_image_name(format, function.image.as_str(), version, branch);

            let spec = Spec {
                name,
                image: image_name,
                environment: all_environment,
                labels: function.labels.clone(),
                limits: function.limits.clone(),
                requests: function.requests.clone(),
                constraints: function.constraints.clone(),
                secrets: function.secrets.clone(),
            };
            let crd = CRD {
                api_version: api_version.to_string(),
                kind: RESOURCE_KIND.to_string(),
                metadata: metadate,
                spec,
            };

            //serialize the object definition to yaml
            //let object_string = format!("{:#?}", crd).replace("CRD", "");
            let object_string =
                serde_json::to_string_pretty(&crd).map_err(|e| Error::Custom(e.to_string()))?;
            //serde_json::to_string(&crd).map_err(|e| Error::Custom(e.to_string()))?;
            objects_string = objects_string + "\n---\n" + object_string.as_str();
        }
    }

    Ok(objects_string)
}

fn generate_knative_v1_serving_service_crd_yaml(
    services: &Services,
    format: BuildFormat,
    api_version: &str,
    namespace: &str,
    branch: &str,
    version: &str,
) -> Result<String> {
    let mut crds: Vec<ServingServiceCRD> = Vec::new();
    let ordered_names = generate_function_order(&services.functions);

    for name in ordered_names {
        let function = services
            .functions
            .get(name.as_str())
            .unwrap_or(&Default::default())
            .to_owned();
        let file_environment = read_files(&function.environment_file)?;

        //combine all environment variables
        let all_environment = compile_environment(
            &Default::default(),
            &function.environment.unwrap_or_default(),
            &file_environment,
        )?;

        let env = order_knative_env(&all_environment);

        //let annotations:HashMap<String,String> = HashMap::new();

        let annotations = function.annotations.clone();

        let image_name = build_image_name(format, function.name.as_str(), version, branch);
        let mut crd = ServingServiceCRD {
            metadata: Metadata {
                name,
                namespace: namespace.to_string(),
                annotations,
            },
            api_version: api_version.to_string(),
            kind: "Service".to_string(),
            ..Default::default()
        };
        let serving_spec = ServingSpecContainersContainerSpec {
            image: image_name,
            env,
            ..Default::default()
        };

        crd.spec
            .serving_service_spec_template
            .template
            .containers
            .push(serving_spec);

        let mut mounts: Vec<VolumeMount> = Vec::new();
        let mut volumes: Vec<Volume> = Vec::new();

        for secret in function.secrets {
            mounts.push(VolumeMount {
                mount_path: "/var/openfaas/secrets/".to_string() + secret.as_str(),
                read_only: true,
                name: secret.clone(),
            });
            volumes.push(Volume {
                name: secret.clone(),
                secret: Secret {
                    secret_name: secret,
                },
            });
        }
        crd.spec.serving_service_spec_template.template.volumes = volumes;
        crd.spec.serving_service_spec_template.template.containers[0].volume_mounts = mounts;
        crds.push(crd);
    }

    let mut objects_string = String::new();

    for crd in crds {
        //Marshal the object definition to yaml
        let object_string =
            serde_json::to_string_pretty(&crd).map_err(|e| Error::Custom(e.to_string()))?;

        objects_string = objects_string + "\n---\n" + object_string.as_str();
    }

    Ok(objects_string)
}

fn generate_function_order(functions: &HashMap<String, Function>) -> Vec<String> {
    let mut function_names = Vec::new();
    for (function_name, _) in functions {
        function_names.push(function_name.clone());
    }
    function_names.sort();
    function_names
}

fn order_knative_env(environment: &HashMap<String, String>) -> Vec<EnvPair> {
    let mut ordered_environment: Vec<String> = Vec::new();
    let mut env_vars: Vec<EnvPair> = Vec::new();

    for (k, _) in environment {
        ordered_environment.push(k.to_string());
    }
    ordered_environment.sort();

    for env_var in ordered_environment {
        env_vars.push(EnvPair {
            name: env_var.clone(),
            value: environment
                .get(env_var.as_str())
                .map(|s| s.to_owned())
                .unwrap_or(String::new()),
        });
    }

    env_vars
}
