use crate::template_pull::TemplatePull;
use crate::template_store::TemplateStore;
use crate::{CommandAppend, State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};

pub(crate) struct Template;

impl CommandAppend for Template {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("template")
                .about(
                    r#"Allows browsing templates from store or pulling custom templates",
	Example: `  faas-cli template pull https://github.com/custom/template
  faas-cli template store list
  faas-cli template store ls
  faas-cli template store pull ruby-http
  faas-cli template store pull openfaas-incubator/ruby-http`"#,
                )
                .arg(
                    Arg::with_name("overwrite")
                        .long("overwrite")
                        .global(true)
                        .help("Overwrite existing templates?"),
                )
                .arg(
                    Arg::with_name("debug")
                        .long("debug")
                        .global(true)
                        .help("Enable debug output"),
                )
                //add subcommands like store
                .subcommand(TemplateStore::append_subcommand())
                .subcommand(TemplatePull::append_subcommand()),
        );
        app
    }
}

impl Template {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(t_args) = args.subcommand_matches("template") {
            TemplateStore::dispatch_command(t_args).await?;
            TemplatePull::dispatch_command(t_args).await?;
            let usage = t_args.usage();

            //println!("{}");

            Err(State::Custom(format!(
                "template command must followed by sub command\n\
                {}\n
                for example:\
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
