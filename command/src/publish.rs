use crate::build::{combine_build_opts, parse_build_args, pull_templates};
use crate::deploy::{merge_map, merge_slice, parse_map};
use crate::fetch_template::DEFAULT_TEMPLATE_REPOSITORY;
use crate::priority::{get_template_url, TEMPLATE_URL_ENVIRONMENT};
use crate::template_pull_stack::{filter_existing_templates, pull_stack_templates};
use crate::template_store_list::DEFAULT_TEMPLATE_STORE;
use crate::validate::validate_language_flag;
use crate::{CommandAppend, State};
use builder::publish::{publish_image, PublishImage};
use clap::{App, Arg, ArgMatches, SubCommand};
use schema::image::{BuildFormat, TBuildFormat};
use stack::schema::{Function, Services};
use stack::stack::parse_yaml_file;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Instant;
use utility::{Error, Result};

pub(crate) struct Publish;

impl CommandAppend for Publish {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("publish")
                .about(r#"Builds and pushes multi-arch OpenFaaS container images using Docker buildx.
                    Most users will want faas-cli build or faas-cli up for development and testing.
                    This command is designed to make releasing and publishing multi-arch container
                       images easier.

                        A stack.yaml file is required, and any images that are built will not be
                       available in the local Docker library. This is due to technical constraints in
                       Docker and buildx. You must use a multi-arch template to use this command with
        correctly configured TARGETPLATFORM and BUILDPLATFORM arguments.

            See also: faas-cli build`,
        Example: `  faas-cli publish --platforms linux/amd64,linux/arm64,linux/arm/7
        faas-cli publish --platforms linux/arm/7 --filter webhook
        faas-cli publish -f go.yml --no-cache --build-arg NPM_VERSION=0.2.2
        faas-cli publish --build-option dev
        faas-cli publish --tag sha
        `"#)
                .args_from_usage(
                    " --platforms [platforms]          'A set of platforms to publish'
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
                .arg(
                    Arg::with_name("tag-extra")
                        .help("Additional extra image tag")
                        .long("tag-extra")
                        .takes_value(true)
                        .global(true)
                        .multiple(true)
                )
        );
        //todo check bash-completion in clap
        // Set bash-completion.
        //_ = buildCmd.Flags().SetAnnotation("handler", cobra.BashCompSubdirsInDir, []string{})

        app
    }
}

impl Publish {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(p_args) = args.subcommand_matches("publish") {
            //pushStack(&services, parallel, tagFormat)
            publish_from_args(p_args).await?;
            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

pub(crate) async fn publish_from_args(p_args: &ArgMatches<'_>) -> utility::Result<()> {
    // let gateway_arg = args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
    let regex = p_args.value_of("regex").unwrap_or("");
    let filter = p_args.value_of("filter").unwrap_or("");
    let envsubst = true; //p_args.is_present("envsubst");
    let yaml_file = p_args.value_of("yaml").ok_or(Error::Custom(
        "yaml fil is required use --yaml (-f) YAML_FILE".to_string(),
    ))?;

    //todo check this
    // let read_template = true; //b_args.is_present("read-template");
    let nocache = p_args.is_present("no-cache");
    let squash = p_args.is_present("squash");
    let shrinkwrap = p_args.is_present("shrinkwrap");
    let quiet_build = p_args.is_present("quiet");
    let disable_stack_pull = p_args.is_present("disable-stack-pull");

    let language = p_args.value_of("lang").unwrap_or_default();
    let platforms = p_args.value_of("platforms").unwrap_or("linux/amd64");
    // let image = b_args.value_of("image").unwrap_or_default();
    // let handler = b_args.value_of("handler").unwrap_or_default();

    let parallel = p_args.value_of("parallel").unwrap_or("1");
    let parallel: usize = parallel.parse().map_err(|_e| {
        Error::Custom("the input parallel must be numeric like --parallel".to_string())
    })?;
    //let function_name = p_args.value_of("name").unwrap_or_default();

    let tag = p_args.value_of("tag").unwrap_or("");

    let mut tag_format: Option<BuildFormat> = None;
    tag_format.set(Some(tag.to_string()))?;

    let build_args: Vec<&str> = p_args.values_of("build-args").unwrap_or_default().collect();
    let build_label: Vec<String> = p_args
        .values_of("build-label")
        .unwrap_or_default()
        .map(|m| m.to_string())
        .collect();
    let build_options: Vec<String> = p_args
        .values_of("build-option")
        .unwrap_or_default()
        .map(|m| m.to_string())
        .collect();
    let copy_extra_paths: Vec<String> = p_args
        .values_of("copy-extra")
        .unwrap_or_default()
        .map(|m| m.to_string())
        .collect();
    let tag_extra: Vec<String> = p_args
        .values_of("tag-extra")
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

    let (_language, _) = validate_language_flag(language);

    let mapped = parse_build_args(&build_args)?;
    let build_label_map = parse_map(&build_label, "build-label")?;

    if parallel < 1 {
        return Err(Error::Custom(
            "the --parallel flag must be great than 0".to_string(),
        ));
    }

    let services = parse_yaml_file(yaml_file, regex, filter, envsubst).await?;

    let temp_url = std::env::var(TEMPLATE_URL_ENVIRONMENT).unwrap_or_default();
    let template_address = get_template_url("", temp_url.as_str(), DEFAULT_TEMPLATE_REPOSITORY);

    pull_templates(template_address.as_str())
        .map_err(|e| Error::Custom(format!("could not pull templates for OpenFaaS: {}", e)))?;

    let mut task = std::process::Command::new("docker");
    task.args(&[
        "buildx",
        "create",
        "--use",
        "--name=multiarch",
        "--node=multiarch",
    ])
    .env("DOCKER_CLI_EXPERIMENTAL", "enabled");

    if quiet_build {
        task.stdout(Stdio::piped());
    }

    let output = task.output().map_err(|e| Error::Custom(e.to_string()))?;

    if !output.status.success() {
        return Err(Error::Custom(format!(
            "non-zero exit code:{}",
            output.status
        )));
    }

    unsafe {
        let out = std::str::from_utf8_unchecked(&output.stdout);
        colour::green!("Created buildx node: {}\n", out)
    }

    if !services.stack_configuration.template_configs.is_empty() && !disable_stack_pull {
        let new_template_infos = filter_existing_templates(
            services.stack_configuration.template_configs.clone(),
            "./template",
        )
        .map_err(|e| {
            Error::Custom(format!(
                "Already pulled templates directory has issue: {}",
                e
            ))
        })?;

        pull_stack_templates(
            new_template_infos,
            DEFAULT_TEMPLATE_STORE,
            yaml_file,
            false,
            false,
        )
        .map_err(|e| {
            Error::Custom(format!(
                "could not pull templates from function yaml file:: {}",
                e
            ))
        })?;
    }

    let errors = publish(
        services,
        parallel,
        build_options,
        mapped,
        build_label_map,
        copy_extra_paths,
        tag_extra,
        platforms.to_string(),
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

    Ok(())
}

async fn publish(
    services: Services,
    queue_depth: usize,
    build_options: Vec<String>,
    build_arg_map: HashMap<String, String>,
    build_label_map: HashMap<String, String>,
    copy_extra: Vec<String>,
    tag_extra: Vec<String>,
    platforms: String,
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

        if queue_depth > 0 {
            if function.skip_build {
                println!("Skipping build of: {}.\n", function.name)
            } else {
                function.name = k;
                let build_label_map = build_label_map.clone();
                let build_arg_map = build_arg_map.clone();
                let copy_extra = copy_extra.clone();
                let build_options = build_options.clone();
                let tag_extra = tag_extra.clone();
                let platforms = platforms.clone();

                let h = tokio::spawn(image_publisher(
                    function,
                    extra,
                    build_options,
                    build_arg_map,
                    build_label_map,
                    copy_extra,
                    tag_extra,
                    platforms,
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
            let platforms = platforms.clone();
            let build_label_map = build_label_map.clone();
            let build_arg_map = build_arg_map.clone();
            let copy_extra = copy_extra.clone();
            let build_options = build_options.clone();
            let tag_extra = tag_extra.clone();

            if function.skip_build {
                println!("Skipping build of: {}.\n", function.name)
            } else {
                function.name = k;
                let mut list = image_publisher(
                    function,
                    extra,
                    build_options,
                    build_arg_map,
                    build_label_map,
                    copy_extra,
                    tag_extra,
                    platforms,
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
    colour::yellow!("[{}] Worker done.\n", index);

    let duration = start.elapsed();
    colour::yellow!("\nTotal build time: {}", duration.as_secs());

    return Ok(errors);
}

async fn image_publisher(
    function: Function,
    extra: Vec<String>,
    build_options: Vec<String>,
    build_arg_map: HashMap<String, String>,
    build_label_map: HashMap<String, String>,
    copy_extra: Vec<String>,
    tag_extra: Vec<String>,
    platforms: String,
    index: usize,
    nocache: bool,
    squash: bool,
    shrinkwrap: bool,
    quiet_build: bool,
    tag_mode: BuildFormat,
) -> Vec<String> {
    let inner_start = Instant::now();
    let mut errors: Vec<String> = Vec::new();
    colour::yellow!("[{}] > Building {}.\n", index, function.name);
    if function.language.is_empty() {
        println!("Please provide a valid language for your function.");
    } else {
        let combined_build_options =
            combine_build_opts(function.build_options.clone(), build_options);
        let combined_build_arg_map = merge_map(function.build_args.clone(), build_arg_map);

        let combined_extra_paths = merge_slice(extra, copy_extra);

        let image_data = PublishImage {
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
            platforms: platforms.as_str(),
            extra_tags: &tag_extra,
        };
        if let Err(e) = publish_image(&image_data).await {
            errors.push(e.to_string());
        }
    }
    let duration = inner_start.elapsed();
    colour::yellow!(
        "[{}] < Building {:?} done in {}s.\n",
        index,
        function,
        duration.as_secs()
    );
    errors
}
