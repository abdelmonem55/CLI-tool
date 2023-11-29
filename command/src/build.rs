use crate::deploy::{merge_map, merge_slice, parse_map};
use crate::faas::check_and_set_default_yaml;
use crate::fetch_template::{fetch_templates, DEFAULT_TEMPLATE_REPOSITORY};
use crate::priority::{get_template_url, TEMPLATE_URL_ENVIRONMENT};
use crate::template_pull_stack::filter_existing_templates;
use crate::template_pull_stack::pull_stack_templates;
use crate::validate::validate_language_flag;
use crate::{CommandAppend, State};
use builder::build::ADDITIONAL_PACKAGE_BUILD_ARGS;
use builder::build::{build_image, BuildImage};
use clap::{App, Arg, ArgMatches, SubCommand};
use schema::image::{BuildFormat, TBuildFormat};
use stack::schema::{Function, Services};
use stack::stack::parse_yaml_file;
use std::collections::HashMap;
use std::time::Instant;
use utility::{Error, Result};
use versioncontol::parse::parse_panned_remote;

pub(crate) fn generate_build_args<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.args_from_usage(
        " --fprocess [fprocess]          'fprocess value to be run as a serverless function by the watchdog'
                          --handler [handler]             'Directory with handler for function, e.g. handler.js'
                          --image   [image]                'Docker image name to build'
                          --lang    [lang]                  'Programming language template'
                          --name    [name]                  'Name of the deployed function'
                          --no-cache                        'Do not use Docker's build cache'
                          --squash                          'Use Docker's squash flag for smaller images [experimental]'
                          --parallel [parallel]             'Build in parallel to depth specified.'
                          --shrinkwrap                      'Just write files to ./build/ folder for shrink-wrapping'
                         --quiet                          'Perform a quiet build, without showing output from Docker'
                         --disable-stack-pull              'Disables the template configuration in the stack.yml'
                         --tag [tag]                         'Override latest tag on function Docker image, accepts latest, sha, branch, or describe'
            ")
        .arg(
            Arg::with_name("build-arg")
                .help("Add a build-arg for Docker (KEY=VALUE)")
                .long("build-arg")
                .short("b")
                .takes_value(true)
                .global(true)
                .multiple(true)
        ).arg(
        Arg::with_name("build-label")
            .help("Add a label for Docker image (LABEL=VALUE)")
            .long("build-label")
            //.short("l")
            .takes_value(true)
            .global(true)
            .multiple(true)
    ).arg(
        Arg::with_name("build-option")
            .help("Set a build option, e.g. dev")
            .long("build-option")
            .takes_value(true)
            .global(true)
            .multiple(true)
    ).arg(
        Arg::with_name("copy-extra")
            .help("Extra paths that will be copied into the function build context")
            .long("copy-extra")
            .takes_value(true)
            .global(true)
            .multiple(true)
    )
    //todo check bash-completion in clap
    // Set bash-completion.
    //_ = buildCmd.Flags().SetAnnotation("handler", cobra.BashCompSubdirsInDir, []string{})
}

pub(crate) struct Build;

impl CommandAppend for Build {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let subcommand = SubCommand::with_name("build").about(
            r#"Builds OpenFaaS function containers either via the supplied YAML config using
the "--yaml" flag (which may contain multiple function definitions), or directly
via flags.`,
	Example: `  faas-cli build -f https://domain/path/myfunctions.yml
  faas-cli build -f ./stack.yml --no-cache --build-arg NPM_VERSION=0.2.2
  faas-cli build -f ./stack.yml --build-option dev
  faas-cli build -f ./stack.yml --tag sha
  faas-cli build -f ./stack.yml --tag branch
  faas-cli build -f ./stack.yml --tag describe
  faas-cli build -f ./stack.yml --filter "*gif*"
  faas-cli build -f ./stack.yml --regex "fn[0-9]_.*"
  faas-cli build --image=my_image --lang=python --handler=/path/to/fn/
                 --name=my_fn --squash
  faas-cli build -f ./stack.yml --build-label org.label-schema.label-name="value"`"#,
        );
        let app = app.subcommand(generate_build_args(subcommand));

        app
    }
}

impl Build {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(b_args) = args.subcommand_matches("build") {
            build_from_args(b_args).await?;
            Err(State::Matched)
            //pushStack(&services, parallel, tagFormat)
        } else {
            Ok(())
        }
    }
}

pub(crate) async fn build_from_args(b_args: &ArgMatches<'_>) -> utility::Result<()> {
    let regex = b_args.value_of("regex").unwrap_or("");
    let filter = b_args.value_of("filter").unwrap_or("");
    let envsubst = true; //b_args.is_present("envsubst");
    let yaml_file = b_args
        .value_of("yaml")
        .unwrap_or(check_and_set_default_yaml().unwrap_or_default());

    //todo check this
    // let read_template = true; //b_args.is_present("read-template");
    let nocache = b_args.is_present("no-cache");
    let squash = b_args.is_present("squash");
    let shrinkwrap = b_args.is_present("shrinkwrap");
    let quiet_build = b_args.is_present("quiet");
    let disable_stack_pull = b_args.is_present("disable-stack-pull");

    let language = b_args.value_of("lang").unwrap_or_default();
    //  let network = b_args.value_of("network").unwrap_or(DEFAULT_NETWORK);
    let image = b_args.value_of("image").unwrap_or_default();
    let handler = b_args.value_of("handler").unwrap_or_default();

    let parallel = b_args.value_of("parallel").unwrap_or("1");
    let parallel: usize = parallel.parse().map_err(|_e| {
        Error::Custom("the input parallel must be numeric like --parallel".to_string())
    })?;
    let function_name = b_args.value_of("name").unwrap_or_default();

    //  let token = b_args.value_of("token").unwrap_or_default();
    // let function_namespace = b_args.value_of("namespace").unwrap_or_default();
    let tag = b_args.value_of("tag").unwrap_or("");

    let mut tag_format: Option<BuildFormat> = None;
    tag_format.set(Some(tag.to_string()))?;

    let build_args: Vec<&str> = b_args.values_of("build-args").unwrap_or_default().collect();
    let build_label: Vec<String> = b_args
        .values_of("build-label")
        .unwrap_or_default()
        .map(|m| m.to_string())
        .collect();
    let build_options: Vec<String> = b_args
        .values_of("build-option")
        .unwrap_or_default()
        .map(|m| m.to_string())
        .collect();
    let copy_extra_paths: Vec<String> = b_args
        .values_of("copy-extra")
        .unwrap_or_default()
        .map(|m| m.to_string())
        .collect();
    //
    // let envvar_opts: Vec<String> = b_args
    //     .values_of("env")
    //     .unwrap_or_default()
    //     .map(|m| m.to_string())
    //     .collect();
    // let annotation_opts: Vec<String> = b_args
    //     .values_of("annotation")
    //     .unwrap_or_default()
    //     .map(|m| m.to_string())
    //     .collect();

    // let constraints: Vec<&str> =
    //     b_args.values_of("constraint").unwrap_or_default().collect();

    let (language, _) = validate_language_flag(language);

    let mapped = parse_build_args(&build_args)?;
    let build_label_map = parse_map(&build_label, "build-label")?;

    if parallel < 1 {
        return Err(Error::Custom(
            "the --parallel flag must be great than 0".to_string(),
        ));
    }

    let services = if !yaml_file.is_empty() {
        parse_yaml_file(yaml_file, regex, filter, envsubst).await?
    } else {
        Default::default()
    };
    let temp_url = std::env::var(TEMPLATE_URL_ENVIRONMENT).unwrap_or_default();
    let template_address = get_template_url("", temp_url.as_str(), DEFAULT_TEMPLATE_REPOSITORY);

    pull_templates(template_address.as_str())
        .map_err(|e| Error::Custom(format!("could not pull templates for OpenFaaS: {}", e)))?;

    if services.functions.is_empty() {
        if image.is_empty() {
            return Err(Error::Custom(
                "please provide a valid --image name for your Docker image".to_string(),
            ));
        }
        if handler.is_empty() {
            return Err(Error::Custom(
                "please provide the full path to your function's handler".to_string(),
            ));
        }
        if function_name.is_empty() {
            return Err(Error::Custom(
                "please provide the deployed --name of your function".to_string(),
            ));
        }
        let image_builder = BuildImage {
            image,
            handler,
            function_name,
            language,
            nocache,
            squash,
            shrinkwrap,
            build_arg_map: &mapped,
            build_options: &build_options,
            tag_mode: tag_format.unwrap(),
            build_label_map: &build_label_map,
            quiet_build,
            copy_extra_paths: &copy_extra_paths,
        };

        build_image(&image_builder).await?;
    } else {
        if !services.stack_configuration.template_configs.is_empty() && !disable_stack_pull {
            let new_temp_infos = filter_existing_templates(
                services.stack_configuration.template_configs.clone(),
                "./template",
            )
            .map_err(|e| {
                Error::Custom(format!(
                    "Already pulled templates directory has issue: {}",
                    e
                ))
            })?;

            pull_stack_templates(new_temp_infos, "", yaml_file, false, false).map_err(|e| {
                Error::Custom(format!(
                    "could not pull templates from function yaml file: {}",
                    e
                ))
            })?;
        }

        let errors = build(
            services,
            parallel,
            build_options,
            mapped,
            build_label_map,
            copy_extra_paths,
            nocache,
            squash,
            shrinkwrap,
            quiet_build,
            tag_format.unwrap_or_default(),
        )
        .await?;

        if !errors.is_empty() {
            let mut error_summary = "Errors received during build:\n".to_string();
            for err in errors {
                error_summary.push('-');
                error_summary.push_str((err + "\n").as_str());
            }

            return Err(Error::Custom(error_summary));
        }
    }

    Ok(())
}

pub(crate) fn parse_build_args(args: &Vec<&str>) -> utility::Result<HashMap<String, String>> {
    let mut mapped = HashMap::new();

    for kvp in args {
        let index = kvp.find('=').ok_or(Error::Custom(
            "each build-arg must take the form key=value".to_string(),
        ))?;

        let (left, right) = kvp.split_at(index + 1);
        //let mut values = vec![left.to_string(), right.to_string()];

        let k = left.trim_matches(' ');
        let v = right.trim_matches(' ');

        if k.is_empty() {
            return Err(Error::Custom(
                "build-arg must have a non-empty key".to_string(),
            ));
        }
        if v.is_empty() {
            return Err(Error::Custom(
                "build-arg must have a non-empty value".to_string(),
            ));
        }

        let val = mapped.get(k).map(|v: &String| v.to_string());
        if k == ADDITIONAL_PACKAGE_BUILD_ARGS && val.is_some() {
            mapped.insert(k.to_string(), val.unwrap() + " " + v);
        } else {
            mapped.insert(k.to_string(), v.to_string());
        }
    }

    Ok(mapped)
}

/// PullTemplates pulls templates from specified git remote. templateURL may be a pinned repository.
pub(crate) fn pull_templates(template_url: &str) -> Result<()> {
    if std::fs::metadata("./template").is_err() {
        colour::yellow!("No templates found in current directory.\n");
        let (template_url, ref_name) = parse_panned_remote(template_url);

        //todo check this
        fetch_templates(template_url.as_str(), ref_name.as_str(), false, false)
            .map_err(|_e| Error::Custom("Unable to download templates from Github.".to_string()))?;
    }
    Ok(())
}

async fn build<'s>(
    services: Services,
    queue_depth: usize,
    build_options: Vec<String>,
    build_arg_map: HashMap<String, String>,
    build_label_map: HashMap<String, String>,
    copy_extra: Vec<String>,
    nocache: bool,
    squash: bool,
    shrinkwrap: bool,
    quiet_build: bool,
    tag_mode: i32,
) -> Result<Vec<String>> {
    let mut errors: Vec<String> = Vec::new();

    let start = Instant::now();

    let mut index: usize = 0;

    let extra_path = services.stack_configuration.copy_extra_paths;
    let mut handles = vec![];
    for (k, mut function) in services.functions {
        let extra = extra_path.clone();

        if queue_depth > 1 {
            if function.skip_build {
                println!("Skipping build of: {}.\n", function.name)
            } else {
                function.name = k;
                let build_label_map = build_label_map.clone();
                let build_arg_map = build_arg_map.clone();
                let copy_extra = copy_extra.clone();
                let build_options = build_options.clone();

                let h = tokio::spawn(image_builder(
                    function,
                    extra,
                    build_options,
                    build_arg_map,
                    build_label_map,
                    copy_extra,
                    index,
                    nocache,
                    squash,
                    shrinkwrap,
                    quiet_build,
                    tag_mode,
                ));
                handles.push(h);
            }
        } else {
            let build_label_map = build_label_map.clone();
            let build_arg_map = build_arg_map.clone();
            let copy_extra = copy_extra.clone();
            let build_options = build_options.clone();

            if function.skip_build {
                println!("Skipping build of: {}.\n", function.name)
            } else {
                function.name = k;
                let mut list = image_builder(
                    function,
                    extra,
                    build_options,
                    build_arg_map,
                    build_label_map,
                    copy_extra,
                    index,
                    nocache,
                    squash,
                    shrinkwrap,
                    quiet_build,
                    tag_mode,
                )
                .await;
                errors.append(&mut list);
            }
        }
        index += 1;
    }
    for handle in handles {
        let mut list = handle.await.map_err(|e| Error::Custom(e.to_string()))?;
        errors.append(&mut list);
    }
    colour::green!("[{}] Worker done.\n", index);

    let duration = start.elapsed();
    colour::green!("\nTotal build time: {}\n", duration.as_secs());

    return Ok(errors);
}

pub(crate) fn combine_build_opts(
    yaml_build_opts: Vec<String>,
    build_flag_build_options: Vec<String>,
) -> Vec<String> {
    merge_slice(yaml_build_opts, build_flag_build_options)
}

async fn image_builder(
    function: Function,
    extra: Vec<String>,
    build_options: Vec<String>,
    build_arg_map: HashMap<String, String>,
    build_label_map: HashMap<String, String>,
    copy_extra: Vec<String>,
    index: usize,
    nocache: bool,
    squash: bool,
    shrinkwrap: bool,
    quiet_build: bool,
    tag_mode: BuildFormat,
) -> Vec<String> {
    let inner_start = Instant::now();
    let mut errors: Vec<String> = Vec::new();
    colour::blue!("[{}] > Building {}.\n", index, function.name);
    if function.language.is_empty() {
        colour::yellow!("Please provide a valid language for your function.\n");
    } else {
        let combined_build_options =
            combine_build_opts(function.build_options.clone(), build_options);
        let combined_build_arg_map = merge_map(function.build_args.clone(), build_arg_map);

        let combined_extra_paths = merge_slice(extra, copy_extra);

        let image_builder = BuildImage {
            image: function.image.as_str(),
            handler: function.handler.as_str(),
            function_name: function.name.as_str(),
            language: function.language.as_str(),
            nocache,
            squash,
            shrinkwrap,
            build_arg_map: &combined_build_arg_map,
            build_options: &combined_build_options,
            tag_mode,
            build_label_map: &build_label_map,
            quiet_build,
            copy_extra_paths: &combined_extra_paths,
        };
        if let Err(e) = build_image(&image_builder).await {
            println!("error pr : {:?}", e);
            errors.push(e.to_string());
        }
    }
    let duration = inner_start.elapsed();
    colour::green!(
        "[{}] < Building {:#?} done in {}s.\n",
        index,
        function,
        duration.as_secs()
    );
    errors
}
