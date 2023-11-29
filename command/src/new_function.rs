use crate::build::pull_templates;
use crate::faas::{DEFAULT_GATEWAY, DEFAULT_SCHEMA_VERSION};
use crate::fetch_template::{DEFAULT_TEMPLATE_REPOSITORY, TEMPLATE_DIRECTORY};
use crate::priority::{
    get_gateway_url, get_template_url, OPENFAAS_URL_ENVIRONMENT, TEMPLATE_URL_ENVIRONMENT,
};
use crate::update_gitignore::update_gitignore;
use crate::validate::validate_language_flag;
use crate::version::print_logo;
use crate::{CommandAppend, State};
use builder::copy_files;
use clap::{App, ArgMatches, SubCommand};
use stack::language_template::{
    is_valid_template, load_language_template, parse_yaml_for_language_template,
};
use stack::schema::Function;
use stack::stack::parse_yaml_data;
use std::io::{ErrorKind, Write};
use utility::faas::types::model::FunctionResources;
use utility::{Error, Result};

pub(crate) struct NewFunction;

impl CommandAppend for NewFunction {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("new")
                .about(r#"The new command creates a new function based upon hello-world in the given
language or type in --list for a list of languages available.`,
	Example: `  faas-cli new chatbot --lang node
  faas-cli new chatbot --lang node --append stack.yml
  faas-cli new text-parser --lang python --quiet
  faas-cli new text-parser --lang python --gateway http://mydomain:8080
  faas-cli new --list`"#)
        ).args_from_usage("[FUNCTION-NAME]
            --lang               [lang]               'Language or template to use'
            --handler            [handler]         'directory the handler will be written to'
            -p ,--prefix             [prefix]                          'Set prefix for the function image'
            --memory-limit       [memory-limit]          'Set a limit for the memory'
            --cpu-limit          [cpu-limit]                      'Set a limit for the CPU'
            --memory-request     [memory-request ]                  'Set a request or the memory'
            --cpu-request        [cpu-request]                      'Set a request value for the CPU'
            --list                                                  'List available languages'
            -a ,--append        [append]                             'Append to existing YAML file'
            -q, --quiet                                               'Skip template notes'
            ");
        app
    }
}

impl NewFunction {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(n_args) = args.subcommand_matches("new") {
            let mut gateway = args
                .value_of("gateway")
                .unwrap_or(DEFAULT_GATEWAY)
                .to_string();
            let mut language = n_args.value_of("lang").unwrap_or_default();
            let mut handler_dir = n_args.value_of("handler").unwrap_or_default();
            let append_file = n_args.value_of("append").unwrap_or_default();
            let function_name = n_args.value_of("FUNCTION-NAME").unwrap_or_default();
            let image_prefix = n_args.value_of("prefix").unwrap_or_default();

            let memory_limit = n_args.value_of("memory-limit").unwrap_or_default();
            let cpu_limit = n_args.value_of("cpu-limit").unwrap_or_default();
            let memory_request = n_args.value_of("memory-request").unwrap_or_default();
            let cpu_request = n_args.value_of("cpu-request").unwrap_or_default();

            let envsubst = n_args.is_present("envsubst");
            let quiet = n_args.is_present("quiet");
            let list = n_args.is_present("list");

            if !list {
                let (lang, _) = validate_language_flag(language);
                language = lang;

                if language.is_empty() && function_name.is_empty() {
                    return Err(State::Custom(
                        "error you must enter language using --lang [LANGUAGE] or
                    set function name using faas new [FUNCTION-NAME]"
                            .to_string(),
                    ));
                }
                if function_name.is_empty() {
                    return Err(State::Custom(
                        "you must supply a function language with the --lang flag".to_string(),
                    ));
                }
                validate_function_name(function_name)?;

                let template_url = std::env::var(TEMPLATE_URL_ENVIRONMENT).unwrap_or("".into());
                let template_address =
                    get_template_url("", template_url.as_str(), DEFAULT_TEMPLATE_REPOSITORY);
                pull_templates(template_address.as_str())?;

                if !is_valid_template(language).await {
                    return Err(State::Custom(format!(
                        "{} is unavailable or not supported",
                        language
                    )));
                }

                let append_mode = !append_file.is_empty();

                let (file_name, output_msg) = if append_mode {
                    if !(append_file.ends_with(".yml") || append_file.ends_with(".yaml")) {
                        return Err(State::Custom(
                            "when appending to a stack the suffix should be .yml or .yaml"
                                .to_string(),
                        ));
                    }

                    if let Err(e) = std::fs::metadata(append_file) {
                        return Err(State::Custom(format!(
                            "unable to find file: {} - {}",
                            append_file, e
                        )));
                    }

                    duplicate_function_name(function_name, append_file, envsubst)?;

                    (
                        append_file.to_string(),
                        format!("Stack file updated: {}\n", append_file),
                    )
                } else {
                    let openfass_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or("".into());
                    gateway = get_gateway_url(
                        gateway.as_str(),
                        DEFAULT_GATEWAY,
                        gateway.as_str(),
                        openfass_url.as_str(),
                    );

                    let file_name = function_name.to_string() + ".yml";
                    (
                        file_name.to_string(),
                        format!("Stack file written: {}\n", file_name),
                    )
                };

                if handler_dir.is_empty() {
                    handler_dir = function_name;
                }

                if std::fs::metadata(handler_dir).is_ok() {
                    return Err(State::Custom(format!(
                        "folder: {} already exists",
                        handler_dir
                    )));
                }

                if std::fs::metadata(&file_name).is_ok() && !append_mode {
                    return Err(State::Custom(format!("file: {} already exists", file_name)));
                }

                #[cfg(target_os = "windows")]
                if let Err(e) = std::fs::create_dir(handler_dir) {
                    return Err(State::Custom(format!(
                        "folder: could not create {} : {}",
                        handler_dir, e
                    )));
                }
                #[cfg(target_os = "unix")]
                {
                    use std::fs::DirBuilder;
                    use std::os::unix::fs::DirBuilderExt;
                    let mut builder = DirBuilder::new();
                    builder.mode(0700).create(handler_dir).map_err(|e| {
                        State::Custom(format!("folder: could not create {} : {}", handler_dir, e))
                    })?;
                }

                colour::green!("Folder: {} created.\n", handler_dir);
                update_gitignore().map_err(|e| {
                    State::Custom(format!(
                        "got unexpected error while updating .gitignore file: {}",
                        e
                    ))
                })?;

                let path_to_template_yaml = format!("./template/{}/template.yml", language);
                if let Err(e) = std::fs::metadata(path_to_template_yaml.as_str()) {
                    if e.kind() == ErrorKind::NotFound {
                        return Err(State::Custom(e.to_string()));
                    }
                };
                let lang_template =
                    parse_yaml_for_language_template(path_to_template_yaml.as_str())
                        .await
                        .map_err(|e| {
                            State::Custom(format!("error reading language template: {}", e))
                        })?;

                let template_handler_folder = if !lang_template.handler_folder.is_empty() {
                    lang_template.handler_folder.as_str()
                } else {
                    "function"
                };

                let from_template_handler = format!(
                    "template/{}/{}",
                    language.trim_end_matches('/'),
                    template_handler_folder.trim_start_matches('/')
                );

                // Create function directory from template.
                let _ = copy_files(from_template_handler.as_str(), handler_dir);
                print_logo();
                colour::green!("\nFunction created in folder: {}\n", handler_dir);

                let mut image_name = format!("{}:latest", function_name);
                let image_prefix_val = get_prefix_value(image_prefix);
                let trim_prefix = image_prefix_val.trim();
                if !trim_prefix.is_empty() {
                    image_name = format!("{}/{}", trim_prefix, image_name)
                }

                let mut function = Function {
                    name: function_name.to_string(),
                    handler: format!("./{}", handler_dir.trim_start_matches("/")),
                    language: language.to_string(),
                    image: image_name,
                    ..Default::default()
                };

                if !memory_limit.is_empty() || !cpu_limit.is_empty() {
                    function.limits = FunctionResources {
                        memory: memory_limit.to_string(),
                        cpu: cpu_limit.to_string(),
                    }
                }
                if !memory_request.is_empty() || !cpu_request.is_empty() {
                    function.requests = FunctionResources {
                        memory: memory_request.to_string(),
                        cpu: cpu_request.to_string(),
                    }
                }

                let yaml_content = prepare_yaml_content(append_mode, gateway.as_str(), &function);
                let mut file;
                #[cfg(target_os = "windows")]
                {
                    let mut options = std::fs::File::options();
                    options.write(true).append(true);
                    file = options
                        .open(format!("./{}", file_name))
                        .map_err(|e| State::Custom(e.to_string()))?;
                }
                //todo fix this
                #[cfg(target_os = "unix")]
                {
                    let mut options = std::fs::File::options();
                    options.write(true).append(true);
                    file = options
                        .open(format!("./{}", file_name))
                        .map_err(|e| State::Custom(e.to_string()))?;
                }

                file.write_all(yaml_content.as_bytes())
                    .map_err(|e| State::Custom(e.to_string()))?;

                colour::green!("{}", output_msg);

                if !quiet {
                    if let Ok(language_template) = load_language_template(language).await {
                        if !language_template.welcome_message.is_empty() {
                            colour::green!("\nNotes:\n");
                            colour::green!("{}\n", language_template.welcome_message)
                        }
                    }
                }
            } else {
                let mut available_templates = Vec::new();
                //var availableTemplates []string
                let template_folders = std::fs::read_dir(TEMPLATE_DIRECTORY)
                    .map_err(|_e| State::Custom("no language templates were found.

                        Download templates:
                                          faas-cli template pull           download the default templates
                                      faas-cli template store list     view the community template store".to_string()))?;

                for file in template_folders {
                    let file = file.map_err(|e| State::Custom(e.to_string()))?.path();
                    if file.is_dir() && file.file_name().is_some() {
                        available_templates
                            .push(file.file_name().unwrap().to_string_lossy().to_string());
                        //availableTemplates = append(availableTemplates, file.Name())
                    }
                }

                colour::green!(
                    "Languages available as templates:\n{}\n",
                    print_available_templates(available_templates)
                );
            }

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

/// provides least-common-denominator validation - i.e. only allows valid Kubernetes services names
fn validate_function_name(function_name: &str) -> Result<()> {
    // Regex for RFC-1123 validation:
    // 	k8s.io/kubernetes/pkg/util/validation/validation.go
    let valid_dns = regex::Regex::new("^[a-z0-9]([-a-z0-9]*[a-z0-9])?$").map_err(|e| {
        Error::Custom(format!(
            "consult the development team and the details are {}",
            e
        ))
    })?;
    if valid_dns.is_match(function_name) {
        Ok(())
    } else {
        Err(Error::Custom(
            "function_name can only contain a-z 0-9 and dashes".to_string(),
        ))
    }
}

fn get_prefix_value(image_prefix: &str) -> String {
    if !image_prefix.is_empty() {
        image_prefix.to_string()
    } else {
        std::env::var("OPENFAAS_PREFIX").unwrap_or_default()
    }
}

fn prepare_yaml_content(append_mode: bool, gateway: &str, function: &Function) -> String {
    let mut yaml_content = format!(
        "  {name}:\nlang: {lang}\nhandler: {handler}\nimage: {image}",
        name = function.name,
        lang = function.language,
        handler = function.handler,
        image = function.image
    );

    if function.requests.clone() != FunctionResources::default() {
        yaml_content += "    requests:\n";
        if !function.requests.cpu.is_empty() {
            yaml_content = format!("{}      cpu: {}\n", yaml_content, function.requests.cpu);
        }
        if !function.requests.memory.is_empty() {
            yaml_content = format!(
                "{}      memory: {}\n",
                yaml_content, function.requests.memory
            );
        }
    }
    if function.limits.clone() != FunctionResources::default() {
        yaml_content += "    limits:\n";
        if !function.requests.cpu.is_empty() {
            yaml_content = format!("{}      cpu: {}\n", yaml_content, function.limits.cpu);
        }
        if !function.requests.memory.is_empty() {
            yaml_content = format!("{}      memory: {}\n", yaml_content, function.limits.memory);
        }
    }
    yaml_content.push('\n');

    if !append_mode {
        yaml_content = format!(
            "version: {} provider:
            name: openfaas
        gateway: {}
        functions:
        {}",
            DEFAULT_SCHEMA_VERSION, gateway, yaml_content
        );
    }

    yaml_content
}

fn print_available_templates(mut available_templates: Vec<String>) -> String {
    let mut result = String::new();
    available_templates.sort();
    // sort.Slice(availableTemplates, func(i, j int) bool {
    //     return availableTemplates[i] < availableTemplates[j]
    // })
    for template in available_templates {
        let str = format!("- {}\n", template);
        result.push_str(str.as_str());
    }
    result
}

fn duplicate_function_name(function_name: &str, append_file: &str, envsubst: bool) -> Result<()> {
    let file_bytes = std::fs::read_to_string(append_file)?;

    let services = parse_yaml_data(file_bytes.as_str(), "", "", envsubst)
        .map_err(|_e| Error::Custom(format!("Error parsing {} yml file", append_file)))?;
    if services.functions.get(function_name).is_some() {
        Err(Error::Custom(format!(
            "Function {} already exists in {} file.
        Cannot have duplicate function names in same yaml file",
            function_name, append_file
        )))
    } else {
        Ok(())
    }
}
