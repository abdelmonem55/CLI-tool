#![allow(dead_code)]
use crate::copy_files;
use exec::command_exec;
use path_clean::PathClean;

use path_abs::PathInfo;
use schema::image::{
    build_image_name, BuildFormat, BRANCH_AND_SHA_FORMAT, DESCRIBE_FORMAT, SHA_FORMAT,
};
use stack::language_template::{is_valid_template, parse_yaml_for_language_template};
use stack::schema::BuildOption;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::atomic::{AtomicU32, Ordering};
use utility::{Error, Result};
use versioncontol::git::{get_git_branch, get_git_describe, get_git_sha};

/// AdditionalPackageBuildArg holds the special build-arg keyname for use with build-opts.
/// Can also be passed as a build arg hence needs to be accessed from commands
pub const ADDITIONAL_PACKAGE_BUILD_ARGS: &str = "ADDITIONAL_PACKAGE";
const DEFAULT_HANDLER_FOLDER: &str = "function";

static DEFAULT_DIR_PERMISSION: AtomicU32 = AtomicU32::new(0700);

pub struct BuildImage<'s> {
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
}

// BuildImage construct Docker image from function parameters
pub async fn build_image(build: &BuildImage<'_>) -> Result<()> {
    if is_valid_template(build.language).await {
        let path_to_template_yaml = format!("./template/{}/template.yml", build.language);
        if let Err(e) = std::fs::metadata(&path_to_template_yaml) {
            if e.kind() == ErrorKind::NotFound {
                return Err(Error::Io(e));
            }
        }
        let lang_temp = parse_yaml_for_language_template(path_to_template_yaml.as_str()).await?;

        let (branch, version) = get_image_tag_values(build.tag_mode)?;
        let image_name = build_image_name(
            build.tag_mode,
            build.image,
            version.as_str(),
            branch.as_str(),
        );
        ensure_handler_path(build.handler).map_err(|_| {
            Error::Custom(format!(
                "building {}, {} is an invalid path",
                image_name, build.handler
            ))
        })?;

        let temp_path = create_build_context(
            build.function_name,
            build.handler,
            build.language,
            is_language_template(build.language),
            lang_temp.handler_folder.as_str(),
            build.copy_extra_paths,
        )?;
        println!(
            "Building: {} with {} template. Please wait..\n",
            image_name, build.language
        );

        if build.shrinkwrap {
            println!("{} shrink-wrapped to {}\n", build.shrinkwrap, temp_path);
            return Ok(());
        }

        let build_opt_options = get_build_option_packages(
            build.build_options,
            build.language,
            &lang_temp.build_options,
        )?;

        let http_proxy = std::env::var("http_proxy").unwrap_or_default();
        let https_proxy = std::env::var("https_proxy").unwrap_or_default();

        let docker_build_val = DockerBuild {
            image: image_name.as_str(),
            version: "",
            no_cache: build.nocache,
            squash: build.squash,
            http_proxy: http_proxy.as_str(),
            https_proxy: https_proxy.as_str(),
            build_arg_map: build.build_arg_map,
            build_opt_packages: &build_opt_options,
            build_label_map: build.build_label_map,

            platforms: "",
            extra_tags: &vec![],
        };

        let (command, args) = get_docker_build_command(&docker_build_val);
        let cmd_args: Vec<&str> = args.iter().map(|a| a.as_str()).collect();

        let res = command_exec(temp_path.as_str(), &command, &cmd_args)?;

        if !res.success() {
            Err(Error::Custom(format!(
                "[{}] received non-zero exit code from build, error: {:?}",
                build.function_name, res
            )))
        } else {
            println!("Image: {} built.\n", image_name);
            Ok(())
        }
    } else {
        Err(Error::Custom(format!(
            "language template: {} not supported, build a custom Dockerfile",
            build.language
        )))
    }
}

fn get_docker_build_command(build: &DockerBuild) -> (&'static str, Vec<String>) {
    let mut flag_slice = build_flag_slice(
        build.no_cache,
        build.squash,
        build.http_proxy,
        build.https_proxy,
        build.build_arg_map,
        build.build_opt_packages,
        build.build_label_map,
    );

    let mut args = vec!["build".to_string()];
    args.append(&mut flag_slice);
    args.push("--tag".to_string());
    args.push(build.image.to_string());
    args.push(".".to_string());

    ("docker", args)
}

#[derive(Debug)]
pub(crate) struct DockerBuild<'s> {
    pub(crate) image: &'s str,
    pub(crate) version: &'s str,
    pub(crate) no_cache: bool,
    pub(crate) squash: bool,
    pub(crate) http_proxy: &'s str,
    pub(crate) https_proxy: &'s str,
    pub(crate) build_arg_map: &'s HashMap<String, String>,
    pub(crate) build_opt_packages: &'s Vec<String>,
    pub(crate) build_label_map: &'s HashMap<String, String>,

    /// Platforms for use with buildx and publish command
    pub(crate) platforms: &'s str,

    /// ExtraTags for published images like :latest
    pub(crate) extra_tags: &'s Vec<String>,
}
pub(crate) fn get_build_option_packages(
    requested_build_options: &Vec<String>,
    language: &str,
    available_build_options: &Vec<BuildOption>,
) -> Result<Vec<String>> {
    let mut build_packages = Vec::new();

    if !requested_build_options.is_empty() {
        let (packages, all_found) = get_packages(available_build_options, requested_build_options);

        build_packages = packages;
        if !all_found {
            return Err(Error::Custom(format!(
                "Error: You're using a build option unavailable for {}.
                    Please check /template/{}/template.yml for supported build options",
                language, language
            )));
        }
    }
    return Ok(build_packages);
}

fn get_packages(
    available_build_packages: &Vec<BuildOption>,
    requested_build_packages: &Vec<String>,
) -> (Vec<String>, bool) {
    let mut packages: Vec<String> = Vec::new();

    for requested in requested_build_packages {
        let mut requested_exits = true;
        for available in available_build_packages {
            if available.name.as_str() == requested {
                let mut pack = available.packages.clone();
                packages.append(&mut pack);
                requested_exits = true;
                break;
            }
        }
        if !requested_exits {
            return (packages, false);
        }
    }

    (de_duplicate(&packages), true)
}

/// createBuildContext creates temporary build folder to perform a Docker build with language template
pub(crate) fn create_build_context(
    function_name: &str,
    handler: &str,
    language: &str,
    use_function: bool,
    handler_folder: &str,
    copy_extra_paths: &Vec<String>,
) -> Result<String> {
    let temp_path = format!("./build/{}/", function_name);
    println!("Clearing temporary build folder: {}\n", temp_path);
    std::fs::remove_dir_all(temp_path.as_str()).map_err(|_| {
        Error::Custom(format!(
            "Error clearing temporary build folder: {}\n",
            temp_path
        ))
    })?;

    let mut function_path = std::path::Path::new(temp_path.as_str()).to_owned();

    if use_function {
        if handler_folder.is_empty() {
            function_path = function_path.join(DEFAULT_HANDLER_FOLDER);
        } else {
            function_path = function_path.join(handler_folder);
        }
    }

    println!(
        "Preparing: {}/ {}\n",
        handler,
        function_path.to_string_lossy()
    );

    if is_running_in_ci() {
        DEFAULT_DIR_PERMISSION.store(0777, Ordering::Relaxed);

        //defaultDirPermissions = 0777;
    }

    #[cfg(target_os = "windows")]
    {
        std::fs::create_dir_all(&function_path).map_err(|e| {
            Error::Custom(format!(
                "Error creating path: {} - {}.\n",
                function_path.to_string_lossy(),
                e
            ))
        })?;
    }

    #[cfg(target_os = "unix")]
    {
        use std::fs::DirBuilder;
        use std::os::unix::fs::DirBuilderExt;
        let permission = DEFAULT_DIR_PERMISSION.load(Ordering::Relaxed);
        let mut builder = DirBuilder::new();
        builder
            .mode(permission)
            .recursive(true)
            .create(function_path)
            .map_err(|e| {
                Error::Custom(format!(
                    "Error creating path: {} - {}.\n",
                    function_path.to_string_lossy(),
                    e
                ))
            })?;
    }
    if use_function {
        let src = format!("./template/{}", language);
        copy_files(src.as_str(), temp_path.as_str())
            .map_err(|e| Error::Custom(format!("Error copying template directory: {}.\n", e)))?;
    }

    let infos = std::fs::read_dir(handler)
        .map_err(|e| Error::Custom(format!("Error reading the handler: {} - {}.\n", handler, e)))?;

    for info in infos {
        let file_name = info?.file_name().to_string_lossy().to_string();
        match file_name.as_str() {
            "build" | "template" => {
                println!("Skipping \"{}\" folder\n", file_name)
            }
            _ => {
                let src = Path::new(handler).join(file_name.as_str()).clean();
                let src = src.to_string_lossy().to_string();
                let dest = function_path.join(file_name.as_str()).clean();
                let dest = dest.to_string_lossy().to_string();
                copy_files(src.as_str(), dest.as_str())?;
            }
        }
    }

    for path in copy_extra_paths {
        let abs = path_in_scope(path.as_str(), "")?;
        // Note that if use_function is false, ie is a `dockerfile` template, then
        // functionPath == tempPath, the docker build context, not the `function` handler folder
        // inside the docker build context
        let dest = function_path
            .join(path)
            .clean()
            .to_string_lossy()
            .to_string();
        copy_files(abs.as_str(), dest.as_str())?;
    }

    return Ok(temp_path);
}

/// pathInScope returns the absolute path to `path` and ensures that it is located within the
/// provided scope. An error will be returned, if the path is outside of the provided scope.
fn path_in_scope(path: &str, scope: &str) -> Result<String> {
    //let scope = std::fs::canonicalize(scope)?.to_string_lossy().to_string();
    //let abs = std::fs::canonicalize(path)?.to_string_lossy().to_string();

    let abs = path_abs::PathAbs::new(path).map_err(|e| Error::Custom(e.to_string()))?;
    let scope = path_abs::PathAbs::new(scope).map_err(|e| Error::Custom(e.to_string()))?;
    let abs = abs.to_string_lossy().to_string();
    let scope = scope.to_string_lossy().to_string();

    if abs == scope {
        Err(Error::Custom(format!(
            "forbidden path appears to equal the entire project: {} ({})",
            abs, scope
        )))
    } else if abs.contains(&scope) {
        Ok(abs.replace("/", r#"\"#))
    } else {
        Err(Error::Custom(format!(
            "forbidden path appears to be outside of the build context: {} ({})",
            scope, abs
        )))
    }
}

// isRunningInCI checks the ENV var CI and returns true if it's set to true or 1
fn is_running_in_ci() -> bool {
    if let Ok(val) = std::env::var("CI") {
        if val == "true" || val == "1" {
            true
        } else {
            false
        }
    } else {
        false
    }
}

/// GetImageTagValues returns the image tag format and component information determined via GIT (branch,version)
pub fn get_image_tag_values(tag_type: BuildFormat) -> Result<(String, String)> {
    let mut version = String::new();
    let mut branch = String::new();
    match tag_type {
        SHA_FORMAT => {
            version = get_git_sha()?;
            if version.is_empty() {
                return Err(Error::Custom(
                    "cannot tag image with Git SHA as this is not a Git repository".to_string(),
                ));
            }
        }
        BRANCH_AND_SHA_FORMAT => {
            branch = get_git_branch()?;
            if branch.is_empty() {
                return Err(Error::Custom(
                    "cannot tag image with Git branch and SHA as this is not a Git repository"
                        .to_string(),
                ));
            }

            version = get_git_sha()?;
            if version.is_empty() {
                return Err(Error::Custom(
                    "cannot tag image with Git SHA as this is not a Git repository".to_string(),
                ));
            }
        }
        DESCRIBE_FORMAT => {
            version = get_git_describe()?;
            if version.is_empty() {
                return Err(Error::Custom(
                    "cannot tag image with Git Tag and SHA as this is not a Git repository"
                        .to_string(),
                ));
            }
        }
        _ => {}
    }

    Ok((branch, version))
}

pub(crate) fn build_flag_slice(
    nocache: bool,
    squash: bool,
    http_proxy: &str,
    https_proxy: &str,
    build_arg_map: &HashMap<String, String>,
    build_option_packages: &Vec<String>,
    build_label_map: &HashMap<String, String>,
) -> Vec<String> {
    let mut space_safe_build_flags = Vec::new();

    if nocache {
        space_safe_build_flags.push("--no-cache".to_string());
    }
    if squash {
        space_safe_build_flags.push("--squash".to_string())
    }

    if !http_proxy.is_empty() {
        space_safe_build_flags.push("--build-arg".to_string());
        space_safe_build_flags.push(format!("http_proxy={}", http_proxy));
    }
    if !https_proxy.is_empty() {
        space_safe_build_flags.push("--build-arg".to_string());
        space_safe_build_flags.push(format!("https_proxy={}", https_proxy));
    }

    for (k, v) in build_arg_map {
        if k.as_str() != ADDITIONAL_PACKAGE_BUILD_ARGS {
            space_safe_build_flags.push("--build-arg".to_string());
            space_safe_build_flags.push(format!("{}={}", k, v));
        } else {
            let mut v = v.split(" ").map(|s| s.to_string()).collect();
            space_safe_build_flags.append(&mut v);
        }
    }

    let mut build_option_packages = build_option_packages.to_owned();
    if !build_option_packages.is_empty() {
        build_option_packages = de_duplicate(&build_option_packages);
        space_safe_build_flags.push("--build-arg".to_string());
        space_safe_build_flags.push(format!(
            "{}={}",
            ADDITIONAL_PACKAGE_BUILD_ARGS,
            build_option_packages.join(" ")
        ));
    }

    for (k, v) in build_label_map {
        space_safe_build_flags.push("--label".to_string());
        space_safe_build_flags.push(format!("{}={}", k, v));
    }

    return space_safe_build_flags;
}

fn de_duplicate(build_opt_packages: &Vec<String>) -> Vec<String> {
    let mut seen_packages: HashMap<String, bool> = HashMap::new();
    let mut ret_packages = Vec::new();

    for package_name in build_opt_packages {
        if seen_packages.get(package_name).is_none() {
            seen_packages.insert(package_name.to_string(), true);
            ret_packages.push(package_name.to_string());
        }
        if let Some(seen) = seen_packages.get(package_name) {
            if !seen {
                seen_packages.insert(package_name.to_string(), true);
                ret_packages.push(package_name.to_string());
            }
        }
    }

    ret_packages
}

pub(crate) fn ensure_handler_path(handler: &str) -> Result<()> {
    std::fs::metadata(handler)
        .map_err(|e| Error::Io(e))
        .map(|_| ())
}

pub(crate) fn is_language_template(language: &str) -> bool {
    language.to_ascii_lowercase() != "dockerfile"
}

#[cfg(test)]
mod tests {
    use crate::build::{
        build_flag_slice, de_duplicate, get_docker_build_command, get_packages,
        is_language_template, path_in_scope, DockerBuild,
    };
    use path_clean::PathClean;
    use stack::schema::BuildOption;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_is_language_template_dockerfile() {
        let language = "Dockerfile";

        let want = false;
        let got = is_language_template(language);
        assert_eq!(got, want);
    }

    #[test]
    fn test_is_language_template_node() {
        let language = "node";
        let want = true;
        let got = is_language_template(language);
        assert_eq!(got, want);
    }
    #[test]
    fn test_get_docker_build_command_no_opts() {
        let docker_build_val = DockerBuild {
            image: "imagename:latest",
            version: "",
            no_cache: false,
            squash: false,
            http_proxy: "",
            https_proxy: "",
            build_arg_map: &Default::default(),
            build_opt_packages: &vec![],
            build_label_map: &Default::default(),
            platforms: "",
            extra_tags: &vec![],
        };

        let want = "build --tag imagename:latest .";
        let want_command = "docker";
        let (command, args) = get_docker_build_command(&docker_build_val);

        let joined: String = args.join(" ");

        assert_eq!(joined, want);
        assert_eq!(command, want_command)
    }

    #[test]
    fn test_get_docker_build_command_no_cache() {
        let docker_build_val = DockerBuild {
            image: "imagename:latest",
            version: "",
            no_cache: true,
            squash: false,
            http_proxy: "",
            https_proxy: "",
            build_arg_map: &Default::default(),
            build_opt_packages: &vec![],
            build_label_map: &Default::default(),
            platforms: "",
            extra_tags: &vec![],
        };

        let want = "build --no-cache --tag imagename:latest .";
        let want_command = "docker";
        let (command, args) = get_docker_build_command(&docker_build_val);

        let joined: String = args.join(" ");

        assert_eq!(joined, want);
        assert_eq!(command, want_command)
    }

    #[test]
    fn test_get_docker_build_command_with_proxies() {
        let docker_build_val = DockerBuild {
            image: "imagename:latest",
            version: "",
            no_cache: false,
            squash: false,
            http_proxy: "http://127.0.0.1:3128",
            https_proxy: "https://127.0.0.1:3128",
            build_arg_map: &Default::default(),
            build_opt_packages: &vec![],
            build_label_map: &Default::default(),
            platforms: "",
            extra_tags: &vec![],
        };

        let want = "build --build-arg http_proxy=http://127.0.0.1:3128 --build-arg https_proxy=https://127.0.0.1:3128 --tag imagename:latest .";
        let want_command = "docker";
        let (command, args) = get_docker_build_command(&docker_build_val);

        let joined: String = args.join(" ");

        assert_eq!(joined, want);
        assert_eq!(command, want_command)
    }
    #[test]
    fn test_get_docker_build_command_with_build_arg() {
        let mut map: HashMap<String, String> = HashMap::new();
        map.insert("USERNAME".to_string(), "admin".to_string());
        map.insert("PASSWORD".to_string(), "1234".to_string());

        let docker_build_val = DockerBuild {
            image: "imagename:latest",
            version: "",
            no_cache: false,
            squash: false,
            http_proxy: "",
            https_proxy: "",
            build_arg_map: &map,
            build_opt_packages: &vec![],
            build_label_map: &Default::default(),
            platforms: "",
            extra_tags: &vec![],
        };

        let (_, values) = get_docker_build_command(&docker_build_val);
        let joined: String = values.join(" ");

        let want_arg1 = "--build-arg USERNAME=admin";
        let want_arg2 = "--build-arg PASSWORD=1234";

        assert!(joined.contains(want_arg1));
        assert!(joined.contains(want_arg2));
    }
    #[test]
    fn test_build_flag_slice() {
        struct BuildFlag {
            title: &'static str,
            nocache: bool,
            squash: bool,
            http_proxy: &'static str,
            https_proxy: &'static str,
            build_arg_map: HashMap<String, String>,
            build_packages: Vec<String>,
            expected_slice: Vec<String>,
            build_label_map: HashMap<String, String>,
        }
        let build_flag_opts = vec![
            BuildFlag {
                title: "no cache & squash only",
                nocache: true,
                squash: true,
                http_proxy: "",
                https_proxy: "",
                build_arg_map: Default::default(),
                build_packages: vec![],
                expected_slice: vec!["--no-cache".into(), "--squash".into()],
                build_label_map: Default::default(),
            },
            BuildFlag {
                title: "no cache & squash & http proxy only",
                nocache: true,
                squash: true,
                http_proxy: "192.168.0.1",
                https_proxy: "",
                build_arg_map: Default::default(),
                build_packages: Default::default(),
                expected_slice: vec![
                    "--no-cache".into(),
                    "--squash".into(),
                    "--build-arg".into(),
                    "http_proxy=192.168.0.1".into(),
                ],
                build_label_map: Default::default(),
            },
            BuildFlag {
                title: "no cache & squash & https-proxy only",
                nocache: true,
                squash: true,
                http_proxy: "",
                https_proxy: "127.0.0.1",
                build_arg_map: Default::default(),
                build_packages: Default::default(),
                expected_slice: vec![
                    "--no-cache".into(),
                    "--squash".into(),
                    "--build-arg".into(),
                    "https_proxy=127.0.0.1".into(),
                ],
                build_label_map: Default::default(),
            },
            BuildFlag {
                title: "no cache & squash & http-proxy & https-proxy only",
                nocache: true,
                squash: true,
                http_proxy: "192.168.0.1",
                https_proxy: "127.0.0.1",
                build_arg_map: Default::default(),
                build_packages: Default::default(),
                expected_slice: [
                    "--no-cache",
                    "--squash",
                    "--build-arg",
                    "http_proxy=192.168.0.1",
                    "--build-arg",
                    "https_proxy=127.0.0.1",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                build_label_map: Default::default(),
            },
            BuildFlag {
                title: "http-proxy & https-proxy only",
                nocache: false,
                squash: false,
                http_proxy: "192.168.0.1",
                https_proxy: "127.0.0.1",
                build_arg_map: Default::default(),
                build_packages: Default::default(),
                expected_slice: [
                    "--build-arg",
                    "http_proxy=192.168.0.1",
                    "--build-arg",
                    "https_proxy=127.0.0.1",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                build_label_map: Default::default(),
            },
            BuildFlag {
                title: "build arg map no spaces",
                nocache: false,
                squash: false,
                http_proxy: "",
                https_proxy: "",
                build_arg_map: [("muppet", "ernie")]
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                build_packages: Default::default(),
                expected_slice: ["--build-arg", "muppet=ernie"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                build_label_map: Default::default(),
            },
            BuildFlag {
                title: "build arg map with spaces",
                nocache: false,
                squash: false,
                http_proxy: "",
                https_proxy: "",
                build_arg_map: [("muppets", "burt and ernie")]
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                build_packages: Default::default(),
                expected_slice: ["--build-arg", "muppets=burt and ernie"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                build_label_map: Default::default(),
            },
            BuildFlag {
                title: "multiple build arg map with spaces",
                nocache: false,
                squash: false,
                http_proxy: "",
                https_proxy: "",
                build_arg_map: [("muppets", "burt and ernie"), ("playschool", "Jemima")]
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                build_packages: Default::default(),
                expected_slice: [
                    "--build-arg",
                    "muppets=burt and ernie",
                    "--build-arg",
                    "playschool=Jemima",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                build_label_map: Default::default(),
            },
            BuildFlag {
                title: "no-cache and squash with multiple build arg map with spaces",
                nocache: true,
                squash: true,
                http_proxy: "",
                https_proxy: "",
                build_arg_map: [("muppets", "burt and ernie"), ("playschool", "Jemima")]
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                build_packages: Default::default(),
                expected_slice: [
                    "--no-cache",
                    "--squash",
                    "--build-arg",
                    "muppets=burt and ernie",
                    "--build-arg",
                    "playschool=Jemima",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
                build_label_map: Default::default(),
            },
            BuildFlag {
                title: "single build-label value",
                nocache: false,
                squash: false,
                http_proxy: "",
                https_proxy: "",
                build_arg_map: [("muppets", "burt and ernie"), ("playschool", "Jemima")]
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                build_packages: Default::default(),
                build_label_map: [("org.label-schema.name", "test function")]
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                expected_slice: [
                    "--build-arg",
                    "muppets=burt and ernie",
                    "--build-arg",
                    "playschool=Jemima",
                    "--label",
                    "org.label-schema.name=test function",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            },
            BuildFlag {
                title: "multiple build-label values",
                nocache: false,
                squash: false,
                http_proxy: "",
                https_proxy: "",
                build_arg_map: [("muppets", "burt and ernie"), ("playschool", "Jemima")]
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect(),
                build_packages: Default::default(),
                build_label_map: [
                    ("org.label-schema.name", "test function"),
                    ("org.label-schema.description", "This is a test function"),
                ]
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
                expected_slice: [
                    "--build-arg",
                    "muppets=burt and ernie",
                    "--build-arg",
                    "playschool=Jemima",
                    "--label",
                    "org.label-schema.name=test function",
                    "--label",
                    "org.label-schema.description=This is a test function",
                ]
                .iter()
                .map(|s| s.to_string())
                .collect(),
            },
        ];

        for test in build_flag_opts {
            println!("test :{}", test.title);
            let flag_slice = build_flag_slice(
                test.nocache,
                test.squash,
                test.http_proxy,
                test.https_proxy,
                &test.build_arg_map,
                &test.build_packages,
                &test.build_label_map,
            );
            println!("{:?}", flag_slice);
            assert_eq!(flag_slice.len(), test.expected_slice.len());
            let is_match = compare_slice_values(&test.expected_slice, &flag_slice);
            assert!(is_match);
        }
    }
    fn compare_slice_values(expected_slice: &Vec<String>, actual_slice: &Vec<String>) -> bool {
        let mut expected_value_map: HashMap<String, usize> = HashMap::new();

        for expected in expected_slice {
            let val = if let Some(val) = expected_value_map.get(expected) {
                *val
            } else {
                0
            };

            expected_value_map.insert(expected.to_string(), val + 1);
        }

        let mut actual_value_map: HashMap<String, usize> = HashMap::new();
        for actual in actual_slice {
            let val = if let Some(val) = actual_value_map.get(actual) {
                *val
            } else {
                0
            };

            actual_value_map.insert(actual.to_string(), val + 1);
        }

        if expected_value_map.len() != actual_value_map.len() {
            return false;
        }

        for (key, expected) in expected_value_map {
            if let Some(val) = actual_value_map.get(&key) {
                if *val != expected {
                    return false;
                }
            } else {
                return false;
            }
        }

        return true;
    }

    #[test]
    fn test_get_packages() {
        struct BuildOpts {
            title: &'static str,
            available_build_options: Vec<BuildOption>,
            requested_build_options: Vec<String>,
            expected_packages: Vec<&'static str>,
        }
        let build_opts = vec![
            BuildOpts {
                title: "Single Option",
                available_build_options: vec![BuildOption {
                    name: "dev".into(),
                    packages: ["jq", "hw", "ke"].iter().map(|s| s.to_string()).collect(),
                }],
                requested_build_options: vec!["dev".into()],
                expected_packages: vec!["jq", "hw", "ke"],
            },
            BuildOpts {
                title: "Two Options one chosen",
                available_build_options: vec![
                    BuildOption {
                        name: "dev".into(),
                        packages: ["jq", "hw", "ke"].iter().map(|s| s.to_string()).collect(),
                    },
                    BuildOption {
                        name: "debug".into(),
                        packages: ["lr", "kt", "jy"].iter().map(|s| s.to_string()).collect(),
                    },
                ],
                requested_build_options: vec!["dev".into()],
                expected_packages: vec!["jq", "hw", "ke"],
            },
            BuildOpts {
                title: "Two Options two chosen",
                available_build_options: vec![
                    BuildOption {
                        name: "dev".into(),
                        packages: ["jq", "hw", "ke"].iter().map(|s| s.to_string()).collect(),
                    },
                    BuildOption {
                        name: "debug".into(),
                        packages: ["lr", "kt", "jy"].iter().map(|s| s.to_string()).collect(),
                    },
                ],
                requested_build_options: vec!["dev".into(), "debug".into()],
                expected_packages: vec!["jq", "hw", "ke", "lr", "kt", "jy"],
            },
            BuildOpts {
                title: "Two Options two chosen with overlaps",
                available_build_options: vec![
                    BuildOption {
                        name: "dev".into(),
                        packages: ["jq", "hw", "ke"].iter().map(|s| s.to_string()).collect(),
                    },
                    BuildOption {
                        name: "debug".into(),
                        packages: ["lr", "jq", "hw"].iter().map(|s| s.to_string()).collect(),
                    },
                ],
                requested_build_options: vec!["dev".into(), "debug".into()],
                expected_packages: vec!["jq", "hw", "ke", "lr"],
            },
        ];

        for test in build_opts {
            println!("test case: {}", test.title);
            let (build_opt_package, _) =
                get_packages(&test.available_build_options, &test.requested_build_options);
            assert_eq!(build_opt_package.len(), test.expected_packages.len());

            let mut found = false;
            for expected_opt_package in test.expected_packages {
                for build_opt_package in &build_opt_package {
                    if build_opt_package == expected_opt_package {
                        found = true;
                        break;
                    }
                    assert!(found);
                }
            }
        }
    }

    #[test]
    fn test_de_duplicate() {
        struct StringOpts {
            title: &'static str,
            input_strings: Vec<String>,
            expected_strings: Vec<String>,
        }
        let string_opts = vec![
            StringOpts {
                title: "No Duplicates",
                input_strings: ["jq", "hw", "ke"].iter().map(|s| s.to_string()).collect(),
                expected_strings: ["jq", "hw", "ke"].iter().map(|s| s.to_string()).collect(),
            },
            StringOpts {
                title: "Duplicates",
                input_strings: ["jq", "hw", "ke", "jq", "hw", "ke"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                expected_strings: ["jq", "hw", "ke"].iter().map(|s| s.to_string()).collect(),
            },
        ];
        for test in string_opts {
            println!("test case {} ....", test.title);
            let strings = de_duplicate(&test.input_strings);
            assert_eq!(strings.len(), test.expected_strings.len());

            for expected in test.expected_strings {
                let mut found = false;
                for unique_string in &strings {
                    if expected == *unique_string {
                        found = true;
                        break;
                    }
                }
                assert!(found)
            }
        }
    }

    #[test]
    fn test_path_in_scope() {
        let root = std::fs::canonicalize(".");
        assert!(root.is_ok());
        let root = root.unwrap();

        struct Case {
            name: &'static str,
            path: &'static str,
            expected_path: PathBuf,
            err: bool,
        }

        let cases = vec![
            Case{
                name:         "can copy folders without any relative path prefix",
                path:         "common/models/prebuilt",
                expected_path: root.join("common/models/prebuilt"),
                err: false
            },
            Case{
                name:         "can copy folders with relative path prefix",
                path:         "common/data/cleaned",
                expected_path: root.join( "common/data/cleaned").clean(),
                err: false
            },
            Case {
                name: "error if path equals the current directory",
                path: "./",
                expected_path: Default::default(),
                err:  true,
            },
            Case{
                name: "error if relative path moves out of the current directory",
                path: "../private",
                expected_path: Default::default(),
                err:  true,
            },
            Case{
                name: "error if absolute path is outside of the current directory",
                path: "/private/common",
                expected_path: Default::default(),
                err:  true,
            },
            Case {
                name: "error if relative path moves out of the current directory, even when hidden in the middle of the path",
                path: "./common/../../private",
                expected_path: Default::default(),
                err:  true,
            }];

        for case in cases {
            println!("test case: {} ...", case.name);
            let ro = root.to_str().unwrap();
            //  let path=case.path.to_string_lossy().to_string();
            let res = path_in_scope(case.path, ro);

            match res {
                Ok(abs) => {
                    assert!(!case.err);
                    let expected = case
                        .expected_path
                        .to_string_lossy()
                        .to_string()
                        .replace("/", "\\");
                    println!("Expected {}", expected);
                    assert_eq!(abs, expected);
                }
                Err(e) => {
                    assert!(case.err);
                    assert!(e.to_string().starts_with("forbidden path"));
                }
            }
        }
    }
}
