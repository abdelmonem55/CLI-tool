use crate::template_store_describe::TemplateStoreDescribe;
use crate::template_store_list::TemplateStoreList;
use crate::template_store_pull::TemplateStorePull;
use crate::{State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};

pub(crate) struct TemplateStore;

impl SubCommandAppend for TemplateStore {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app =
            SubCommand::with_name("store")
                .about(r#"This command provides the list of the templates from the official store by default`,
	Example: `  faas-cli template store list --verbose
  faas-cli template store ls -v
  faas-cli template store pull ruby-http
  faas-cli template store pull --url=https://raw.githubusercontent.com/openfaas/store/master/templates.json`"#)
            //     .arg(
            //         Arg::with_name("url")
            //             .long("url")
            //             .short("u")
            //             .global(true)
            //             .takes_value(true)
            //             .default_value(DEFAULT_STORE)
            //             .help("Path to YAML file describing function(s)"),
            //     )
             .arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .global(true)
                    .help("verbose the output"),
            )


                //add subcommands like list, ls and pull
                .subcommand(TemplateStoreList::append_subcommand())
                .subcommand(TemplateStoreDescribe::append_subcommand())
                .subcommand(TemplateStorePull::append_subcommand());
        app
    }
}

impl TemplateStore {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(s_args) = args.subcommand_matches("store") {
            TemplateStoreList::dispatch_command(s_args).await?;
            TemplateStoreDescribe::dispatch_command(s_args).await?;
            TemplateStorePull::dispatch_command(s_args).await?;
            let usage = s_args.usage();
            Err(State::Custom(format!(
                "template store must followed by sub command for example:\n\
             {}
  faas-cli template store list
  faas-cli template store ls
  faas-cli template store pull ruby-http
  faas-cli template store pull openfaas-incubator/ruby-http",
                usage
            )))
        } else {
            Ok(())
        }
    }
}
