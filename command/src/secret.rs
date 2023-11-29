use crate::secret_create::SecretCreate;
use crate::secret_list::SecretList;
use crate::secret_remove::SecretRemove;
use crate::secret_update::SecretUpdate;
use crate::{CommandAppend, State, SubCommandAppend};
use clap::{App, ArgMatches, SubCommand};

pub(crate) struct Secret;

impl CommandAppend for Secret {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("secret")
                .about(r#"Manage function secrets"#)
                //add subcommands like store
                .subcommand(SecretList::append_subcommand()) // .subcommand(TemplatePull::append_subcommand()),
                .subcommand(SecretCreate::append_subcommand())
                .subcommand(SecretRemove::append_subcommand())
                .subcommand(SecretUpdate::append_subcommand()),
        );
        app
    }
}

impl Secret {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(s_args) = args.subcommand_matches("secret") {
            // TemplateStore::dispatch_command(t_args).await?;
            SecretList::dispatch_command(s_args).await?;
            SecretCreate::dispatch_command(s_args).await?;
            SecretRemove::dispatch_command(s_args).await?;
            SecretUpdate::dispatch_command(s_args).await?;

            let usage = s_args.usage();

            Err(State::Custom(format!(
                "template command must followed by sub command\n\
                {}\n
                for example:\
  Example: `faas-cli secret list | update | delete | create
faas-cli secret list --gateway=http://127.0.0.1:8080
enter :faas-cli secret --help
",
                usage
            )))
        } else {
            Ok(())
        }
    }
}
