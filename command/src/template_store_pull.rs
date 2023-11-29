use crate::priority::{get_template_store_url, TEMPLATE_STORE_URL_ENVIRONMENT};
use crate::store::DEFAULT_STORE;
use crate::template_pull::run_template_pull;
use crate::template_store_list::{get_template_info, DEFAULT_TEMPLATE_STORE};
use crate::{State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};

pub(crate) struct TemplateStorePull;

impl SubCommandAppend for TemplateStorePull {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app =
            SubCommand::with_name("pull")
                .about(r#"Pull templates from store supported by openfaas or openfaas-incubator organizations or your custom store`,
	Example: `  faas-cli template store pull ruby-http
  faas-cli template store pull go --debug
  faas-cli template store pull openfaas/go --overwrite
  faas-cli template store pull golang-middleware --url https://raw.githubusercontent.com/openfaas/store/master/templates.json`"#)
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

impl TemplateStorePull {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(p_args) = args.subcommand_matches("pull") {
            let template_name = p_args.value_of("TEMPLATE_NAME")
                .ok_or(State::Custom(
                    format!("\nNeed to specify one of the store templates, check available ones by running the command:\nfaas-cli template store list")))?;
            let overwrite = p_args.is_present("overwrite");
            let debug = p_args.is_present("debug");

            let template_store_url = p_args.value_of("url").unwrap_or(DEFAULT_STORE);
            // let yaml_file = args
            //     .value_of("yaml")
            //     .unwrap_or(check_and_set_default_yaml().unwrap_or_default());
            // let gateway = args.value_of("gateway").ok_or(State::Custom(format!(
            //     "you must set gateway using \
            //  --gateway, -g http://host"
            // )))?;

            // let gateway = gateway.trim();
            // let openfass_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or("".into());
            // let gateway = get_gateway_url(gateway, DEFAULT_GATEWAY, "", openfass_url.as_str());
            // remove_auth_config(gateway.as_str())?;
            let env_template_store =
                std::env::var(TEMPLATE_STORE_URL_ENVIRONMENT).unwrap_or_default();
            let store_url = get_template_store_url(
                template_store_url,
                env_template_store.as_str(),
                DEFAULT_TEMPLATE_STORE,
            );

            let store_templates = get_template_info(store_url.as_str()).await.map_err(|e| {
                return State::Custom(format!("error while fetching templates from store: {}", e));
            })?;

            let mut found = true;
            for store_template in store_templates {
                //like hub.docker.io/my-image
                let source_name =
                    format!("{}/{}", store_template.source, store_template.template_name);

                if template_name == store_template.template_name || template_name == source_name {
                    run_template_pull("", overwrite, debug).map_err(|e| {
                        State::Custom(format!(
                            "error while pulling template: {} : {}",
                            store_template.template_name, e
                        ))
                    })?;
                    found = true;
                    break;
                }
            }

            if !found {
                Err(State::Custom(format!(
                    "template with name: `{}` does not exist in the repo",
                    template_name
                )))
            } else {
                Err(State::Matched)
            }
        } else {
            Ok(())
        }
    }
}
