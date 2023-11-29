use crate::{CommandAppend, State};
use clap::{App, ArgMatches, SubCommand};

use crate::faas::check_and_set_default_yaml;
use builder::build::get_image_tag_values;
use colour::yellow;
use schema::image::{build_image_name, BuildFormat, TBuildFormat, DEFAULT_FORMAT};
use stack::schema::{Function, Services};
use stack::stack::parse_yaml_file;
use std::collections::HashMap;
use utility::Error;

pub(crate) struct Push;

impl CommandAppend for Push {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("push")
                .about(r#"Pushes the OpenFaaS function container image(s) defined in the supplied YAML
config to a remote repository.

These container images must already be present in your local image cache.`,

	Example: `  faas-cli push -f https://domain/path/myfunctions.yml
  faas-cli push -f ./stack.yml
  faas-cli push -f ./stack.yml --parallel 4
  faas-cli push -f ./stack.yml --filter "*gif*"
  faas-cli push -f ./stack.yml --regex "fn[0-9]_.*"
  faas-cli push -f ./stack.yml --tag sha
  faas-cli push -f ./stack.yml --tag branch
  faas-cli push -f ./stack.yml --tag describe' "#)
                .args_from_usage(
                    "--parallel [parallel]  'Push images in parallel to depth specified.'
                           --tag [tag]       'Override latest tag on function Docker image, accepts 'latest', 'sha', 'branch', 'describe'
            ")
        );
        app
    }
}

impl Push {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(ps_args) = args.subcommand_matches("push") {
            push_from_args(ps_args).await?;
            Err(State::Matched)
            //pushStack(&services, parallel, tagFormat)
        } else {
            Ok(())
        }
    }
}

pub(crate) async fn push_from_args(ps_args: &ArgMatches<'_>) -> utility::Result<()> {
    // let gateway = args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
    let regex = ps_args.value_of("regex").unwrap_or("");
    let filter = ps_args.value_of("filter").unwrap_or("");
    let yaml_file = ps_args
        .value_of("yaml")
        .unwrap_or(check_and_set_default_yaml().unwrap_or_default());
    let envsubst = true; //ps_args.is_present("envsubst");
    let parallel = ps_args.value_of("parallel").unwrap_or("1");
    let tag = ps_args.value_of("tag").unwrap_or("");
    let parallel: usize = parallel
        .parse()
        .map_err(|_e| Error::Custom(format!("{} not valid integer positive number", parallel)))?;

    let mut tag_format: Option<BuildFormat> = None;
    tag_format.set(Some(tag.to_string()))?;

    // let openfass_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or_default();
    // let gateway_address = get_gateway_url(gateway,DEFAULT_GATEWAY,"",openfass_url.as_str());
    //
    // let cli_auth = ClientAuthE::new(token,gateway_address.as_str())?;

    //transport := GetDefaultCLITransport(tlsInsecure, &commandTimeout)
    //client, err := proxy.NewClient(cliAuth, gatewayAddress, transport, &commandTimeout)
    //let mut client =cli_auth.get_client(gateway_address.as_str())?;

    let services = if !yaml_file.is_empty() {
        parse_yaml_file(yaml_file, regex, filter, envsubst).await?
    } else {
        Services::default()
    };
    if !services.functions.is_empty() {
        let mut invalid_images = validate_images(&services.functions);
        if !invalid_images.is_empty() {
            invalid_images.push("\n- ".to_string());
            return Err(Error::Custom(format!("
                                      Unable to push one or more of your functions to Docker Hub:
                                      - ` + {:?} + `

                                      You must provide a username or registry prefix to the Function's image such as user1/function1",invalid_images)));
        }
        push_stack(&services, parallel, tag_format.unwrap()).await?;
        Ok(())
    } else {
        Err(Error::Custom(
            "you must supply a valid YAML file".to_string(),
        ))
    }
}

async fn push_stack(
    services: &Services,
    queue_depth: usize,
    tag_mode: BuildFormat,
) -> utility::Result<()> {
    let mut tag_mode = tag_mode;

    let mut index = -1;
    let mut handles = vec![];

    let mut push_func = move |function: Function, index: i32| -> utility::Result<()> {
        let res = get_image_tag_values(tag_mode.clone());

        let (branch, sha) = if let Ok((b, v)) = res {
            (b, v)
        } else {
            tag_mode = DEFAULT_FORMAT;
            (String::new(), String::new())
        };
        let image_name = build_image_name(
            tag_mode,
            function.image.as_str(),
            sha.as_str(),
            branch.as_str(),
        );

        let str = format!(
            "[{}] > Pushing {} [{}].\n",
            index, function.name, image_name
        );
        yellow!("{}", str.as_str());

        if function.image.is_empty() {
            println!("Please provide a valid Image value in the YAML file.")
        } else if function.skip_build {
            println!("Skipping {}\n", function.name);
        } else {
            push_image(image_name.as_str())?;
            let str = format!(
                "[{}] < Pushing {} [{}] done.\n",
                index, function.name, image_name
            );
            yellow!("{}", str.as_str());
        }
        Ok(())
    };
    for (_, function) in services.functions.clone() {
        index += 1;
        if queue_depth != 0 {
            let h = tokio::spawn(async move { push_func(function, index) });
            handles.push(h);
        } else {
            push_func(function, index)?;
        }
    }
    for handle in handles {
        handle.await.map_err(|e| Error::Custom(e.to_string()))??;
    }

    let str = format!("[{}] Worker done.\n", index);
    yellow!("{}", str.as_str());
    Ok(())
}

fn validate_images(functions: &HashMap<String, Function>) -> Vec<String> {
    let mut images = Vec::new();

    for (name, function) in functions {
        if function.skip_build && !function.image.contains('/') {
            images.push(name.to_string());
        }
    }
    images
}

fn push_image(image: &str) -> utility::Result<()> {
    exec::command("./", vec!["docker", "push", image])
}
