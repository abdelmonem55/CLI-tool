use crate::priority::{get_template_store_url, TEMPLATE_STORE_URL_ENVIRONMENT};
use crate::store::DEFAULT_STORE;
use crate::template_store_list::{get_template_info, TemplateInfo, DEFAULT_TEMPLATE_STORE};
use crate::{State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};
use utility::{Error, Result};

pub(crate) struct TemplateStoreDescribe;

impl SubCommandAppend for TemplateStoreDescribe {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app =
            SubCommand::with_name("describe")
                .alias("ls")
                .about(r#"Describe the template by outputting all the fields that the template struct has`,
	Example: `  faas-cli template store describe golang-http
  faas-cli template store describe haskell --url https://raw.githubusercontent.com/custom/store/master/templates.json``"#)
                .arg(
                    Arg::with_name("TEMPLATE_NAME")
                        .help("platform name")
                        .index(1)
                )
            .arg(
                Arg::with_name("url")
                    .long("url")
                    .short("u")
                    .global(true)
                    .takes_value(true)
                    .default_value(DEFAULT_STORE)
                    .help("Path to YAML file describing function(s)"),
            );
        app
    }
}

impl TemplateStoreDescribe {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(d_args) = args.subcommand_matches("describe") {
            let template_store_url = d_args.value_of("url").unwrap_or(DEFAULT_STORE);
            let template = d_args.value_of("TEMPLATE_NAME")
                .ok_or(State::Custom(
                    format!("\nNeed to specify one of the store templates, check available ones by running the command:\n\
                    faas-cli template store list")))?;

            let env_template_repo_store =
                std::env::var(TEMPLATE_STORE_URL_ENVIRONMENT).unwrap_or_default();
            let store_url = get_template_store_url(
                template_store_url,
                env_template_repo_store.as_str(),
                DEFAULT_TEMPLATE_STORE,
            );

            let template_info = get_template_info(store_url.as_str())
                .await
                .map_err(|e| State::Custom(format!("error while template info: {}", e)))?;

            let store_template = check_existing_template(template_info, template)?;
            let template_info = format_template_output(store_template);
            colour::green!("{}", template_info);

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

fn check_existing_template(
    store_template: Vec<TemplateInfo>,
    template: &str,
) -> Result<TemplateInfo> {
    for store_template in store_template {
        let source_name = format!("{}/{}", store_template.source, store_template.template_name);
        if template == store_template.template_name || template == source_name {
            return Ok(store_template);
        }
    }
    Err(Error::Custom(format!(
        "template with name: `{}` does not exist in the store",
        template
    )))
}

fn format_template_output(store_template: TemplateInfo) -> String {
    let mut out = format!("Name:\t{}\n", store_template.template_name);
    let fmt = format!("Platform:\t{}\n", store_template.platform);
    out.push_str(fmt.as_str());
    format!("Language:\t{}\n", store_template.language);
    out.push_str(fmt.as_str());
    format!("Source:\t{}\n", store_template.source);
    out.push_str(fmt.as_str());
    format!("Description:\t{}\n", store_template.description);
    out.push_str(fmt.as_str());
    format!("Repository:\t{}\n", store_template.repository);
    out.push_str(fmt.as_str());
    format!("Official Template:\t{}\n", store_template.official);
    out.push_str(fmt.as_str());

    out
}
