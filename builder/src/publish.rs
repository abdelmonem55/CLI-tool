use crate::build::{
    build_flag_slice, create_build_context, ensure_handler_path, get_build_option_packages,
    get_image_tag_values, is_language_template, DockerBuild,
};
use exec::command_exec;
use schema::image::{build_image_name, BuildFormat};
use stack::language_template::{is_valid_template, parse_yaml_for_language_template};
use std::collections::HashMap;
use std::io::ErrorKind;
use utility::{Error, Result};

pub struct PublishImage<'s> {
    pub image: &'s str,
    pub handler: &'s str,
    pub function_name: &'s str,
    pub language: &'s str,
    pub nocache: bool,
    pub squash: bool,
    pub shrinkwrap: bool,
    pub build_arg_map: &'s HashMap<String, String>,
    pub build_options: &'s Vec<String>,
    pub tag_mode: BuildFormat,
    pub build_label_map: &'s HashMap<String, String>,
    pub quiet_build: bool,
    pub copy_extra_paths: &'s Vec<String>,
    pub platforms: &'s str,
    pub extra_tags: &'s Vec<String>,
}

pub async fn publish_image(publish: &PublishImage<'_>) -> Result<()> {
    if is_valid_template(publish.language).await {
        let path_to_template_yaml = format!("./template/{}/template.yml", publish.language);
        if let Err(e) = std::fs::metadata(&path_to_template_yaml) {
            if e.kind() == ErrorKind::NotFound {
                return Err(Error::Io(e));
            }
        }
        let lang_template = parse_yaml_for_language_template(path_to_template_yaml.as_str())
            .await
            .map_err(|e| Error::Custom(format!("error reading language template: {}", e)))?;

        let (branch, version) = get_image_tag_values(publish.tag_mode)?;

        let image_name = build_image_name(
            publish.tag_mode,
            publish.image,
            version.as_str(),
            branch.as_str(),
        );
        ensure_handler_path(publish.handler).map_err(|_e| {
            Error::Custom(format!(
                "building {}, {} is an invalid path",
                image_name, publish.handler
            ))
        })?;

        let temp_path = create_build_context(
            publish.function_name,
            publish.handler,
            publish.language,
            is_language_template(publish.language),
            lang_template.handler_folder.as_str(),
            publish.copy_extra_paths,
        )?;

        println!(
            "Building: {} with {} template. Please wait..\n",
            image_name, publish.language
        );

        if publish.shrinkwrap {
            format!(
                "{} shrink-wrapped to {}\n",
                publish.function_name, temp_path
            );
            return Ok(());
        }

        let build_opt_packages = get_build_option_packages(
            publish.build_options,
            publish.language,
            &lang_template.build_options,
        )?;

        let http_proxy = std::env::var("http_proxy").unwrap_or_default();
        let https_proxy = std::env::var("https_proxy").unwrap_or_default();
        let docker_build_val = DockerBuild {
            image: image_name.as_str(),
            version: "",
            no_cache: publish.nocache,
            squash: publish.squash,
            http_proxy: http_proxy.as_str(),
            https_proxy: https_proxy.as_str(),
            build_arg_map: publish.build_arg_map,
            build_opt_packages: &build_opt_packages,
            build_label_map: publish.build_label_map,
            platforms: publish.platforms,
            extra_tags: publish.extra_tags,
        };

        let (command, args) = get_docker_buildx_command(docker_build_val);
        println!("Publishing with command: {} {:?}\n", command, args);

        let cmd_args: Vec<&str> = args.iter().map(|a| a.as_str()).collect();

        let res = command_exec(temp_path.as_str(), &command, &cmd_args)?;

        if !res.success() {
            Err(Error::Custom(format!(
                "[{}] received non-zero exit code from build, error: {:?}",
                publish.function_name, res
            )))
        } else {
            println!("Image: {} published.\n", image_name);
            Ok(())
        }
    } else {
        Err(Error::Custom(format!(
            "language template: {} not supported, build a custom Dockerfile",
            publish.language
        )))
    }
}

fn get_docker_buildx_command(build: DockerBuild) -> (&'static str, Vec<String>) {
    let mut flag_slice = build_flag_slice(
        build.no_cache,
        build.squash,
        build.http_proxy,
        build.https_proxy,
        build.build_arg_map,
        build.build_opt_packages,
        build.build_label_map,
    );

    /// pushOnly defined at https://github.com/docker/buildx
    const PUSH_ONLY: &str = "--output=type=registry,push=true";
    let mut args = vec![
        "buildx".to_string(),
        "build".to_string(),
        "--progress=plain".to_string(),
        "--platform=".to_string() + build.platforms,
        PUSH_ONLY.to_string(),
    ];

    args.append(&mut flag_slice);
    args.push("--tag".to_string());
    args.push(build.image.to_string());
    args.push(".".to_string());

    for t in build.extra_tags {
        let tag = if let Some(index) = build.image.rfind(':') {
            apply_tag(index, build.image, t)
        } else {
            apply_tag(build.image.len() - 1, build.image, t)
        };
        args.push("--tag".to_string());
        args.push(tag);
    }

    ("docker", args)
}

fn apply_tag(index: usize, base_image: &str, tag: &str) -> String {
    // return fmt.Sprintf("%s:%s", baseImage[:index], tag);
    let (arr, _) = base_image.split_at(index);
    format!("{}:{}", arr, tag)
}
