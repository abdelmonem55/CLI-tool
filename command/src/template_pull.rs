use crate::fetch_template::{pull_template, DEFAULT_TEMPLATE_REPOSITORY};
use crate::priority::{get_template_url, TEMPLATE_URL_ENVIRONMENT};
use crate::template_pull_stack::TemplatePullStack;
use crate::{State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};
use utility::Result;

pub(crate) struct TemplatePull;

impl SubCommandAppend for TemplatePull {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app =
            SubCommand::with_name("pull")
                .about(r#"Downloads templates from the specified git repo specified by [REPOSITORY_URL], and copies the 'template'
directory from the root of the repo, if it exists.

[REPOSITORY_URL] may specify a specific branch or tag to copy by adding a URL fragment with the branch or tag name.
	`,
	Example: `
  faas-cli template pull https://github.com/openfaas/templates
  faas-cli template pull https://github.com/openfaas/templates#1.0
`,"#)
                .arg(
                    Arg::with_name("REPOSITORY_URL")
                        .help("repository url which specify a specific branch or tag to copy by adding a URL fragment with the branch or tag name")
                        .index(1)
                )
                .subcommand(TemplatePullStack::append_subcommand());
        app
    }
}

impl TemplatePull {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(p_args) = args.subcommand_matches("pull") {
            TemplatePullStack::dispatch_command(p_args).await?;

            let repository = p_args.value_of("REPOSITORY_URL").unwrap_or_default();
            let overwrite = p_args.is_present("overwrite");
            let debug = p_args.is_present("debug");
            run_template_pull(repository, overwrite, debug)?;

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

pub(crate) fn run_template_pull(repository: &str, overwrite: bool, pull_debug: bool) -> Result<()> {
    let env_url = std::env::var(TEMPLATE_URL_ENVIRONMENT).unwrap_or_default();
    let repository = get_template_url(repository, env_url.as_str(), DEFAULT_TEMPLATE_REPOSITORY);
    pull_template(repository.as_str(), overwrite, pull_debug)
}

#[allow(dead_code)]
pub(crate) fn pull_debug_print(message: &str, pull_debug: bool) {
    if pull_debug {
        println!("{}", message);
    }
}
