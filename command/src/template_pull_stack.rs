use crate::faas::check_and_set_default_yaml;
use crate::fetch_template::pull_template;
use crate::template_pull::run_template_pull;
use crate::{State, SubCommandAppend};
use clap::{App, ArgMatches, SubCommand};
use stack::schema::{Configuration, TemplateSource};
use std::io::ErrorKind;
use utility::{Error, Result};

pub(crate) struct TemplatePullStack;

impl SubCommandAppend for TemplatePullStack {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app = SubCommand::with_name("stack").about(
            r#"Downloads templates specified in the function yaml file, in the current directory
	`,
	Example: `
  faas-cli template pull stack
  faas-cli template pull stack -f myfunction.yml
  faas-cli template pull stack -r custom_repo_name`"#,
        );
        app
    }
}

impl TemplatePullStack {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(s_args) = args.subcommand_matches("stack") {
            let yaml_file = args
                .value_of("yaml")
                .unwrap_or(check_and_set_default_yaml().unwrap_or_default());
            let repository = s_args.value_of("REPOSITORY_URL").unwrap_or_default();
            let overwrite = s_args.is_present("overwrite");
            let debug = s_args.is_present("debug");

            let template_config = load_template_config(yaml_file)?;
            pull_stack_templates(template_config, repository, yaml_file, overwrite, debug)?;

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

pub(crate) fn pull_stack_templates(
    template_info: Vec<TemplateSource>,
    repository: &str,
    yaml_file: &str,
    overwrite: bool,
    pull_debug: bool,
) -> Result<()> {
    for val in template_info {
        colour::green!(
            "Pulling template: {} from configuration file: {}\n",
            val.name,
            yaml_file
        );
        if val.source.is_empty() {
            run_template_pull(repository, overwrite, pull_debug)?;
        } else {
            pull_template(val.source.as_str(), overwrite, pull_debug)?;
        }
    }
    Ok(())
}

fn load_template_config(yaml_file: &str) -> Result<Vec<TemplateSource>> {
    let stack_config = read_stack_config(yaml_file)?;

    Ok(stack_config.stack_config.template_configs)
}

fn read_stack_config(yaml_file: &str) -> Result<Configuration> {
    let config_field_bytes = std::fs::read_to_string(yaml_file)
        .map_err(|e| Error::Custom(format!("Error while reading files {}", e)))?;

    let config_field: Configuration = serde_yaml::from_str(config_field_bytes.as_str())
        .map_err(|e| Error::Custom(format!("Error while reading configuration: {}", e)))?;

    if config_field.stack_config.template_configs.is_empty() {
        Err(Error::Custom(
            "Error while reading configuration: no template repos currently configured".to_string(),
        ))
    } else {
        Ok(config_field)
    }
}

#[allow(dead_code)]
fn find_template<'s>(
    template_info: &'s Vec<TemplateSource>,
    custom_name: &'s str,
) -> Option<&'s TemplateSource> {
    for val in template_info {
        if val.name == custom_name {
            return Some(val);
        }
    }
    None
}

// /// filter templates which are already available on filesystem
// pub(crate) fn filter_existing_templates<'s>(
//     template_info: &'s Vec<TemplateSource>,
//     templates_dir: &str,
// ) -> Result<Vec<&'s TemplateSource>> {
//     let mut templates: Vec<&TemplateSource> = Vec::new();
//     for info in template_info {
//         let template_path = format!("{}/{}", templates_dir, info.name);
//         if let Err(e) = std::fs::metadata(template_path) {
//             if e.kind() == ErrorKind::NotFound {
//                 templates.push(info);
//             } else {
//                 return Err(Error::Io(e));
//             }
//         }
//     }
//
//     Ok(templates)
// }

/// filter templates which are already available on filesystem
pub(crate) fn filter_existing_templates(
    template_info: Vec<TemplateSource>,
    templates_dir: &str,
) -> Result<Vec<TemplateSource>> {
    let mut templates: Vec<TemplateSource> = Vec::new();
    for info in template_info {
        let template_path = format!("{}/{}", templates_dir, info.name);
        if let Err(e) = std::fs::metadata(template_path) {
            if e.kind() == ErrorKind::NotFound {
                templates.push(info);
            } else {
                return Err(Error::Io(e));
            }
        }
    }

    Ok(templates)
}
