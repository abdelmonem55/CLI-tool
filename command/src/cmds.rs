use crate::auth::Auth;
use crate::build::Build;
use crate::cloud::Cloud;
use crate::deploy::Deploy;
use crate::describe::Describe;
use crate::faas::Faas;
use crate::generate::Generate;
use crate::invoke::Invoke;
use crate::list::List;
use crate::login::Login;
use crate::logout::Logout;
use crate::logs::Logs;
use crate::namespaces::Namespaces;
use crate::new_function::NewFunction;
use crate::publish::Publish;
use crate::push::Push;
use crate::registry_login::RegistryLogin;
use crate::remove::Remove;
use crate::secret::Secret;
use crate::store::Store;
use crate::template::Template;
use crate::up::Up;
use crate::version::Version;
use crate::{CommandAppend, State};
use clap::{App, AppSettings, ArgMatches};
use utility::{Error, Result};

const WELCOME_MSG: &str = r#"  ___                   _____           ____
 / _ \ _ __   ___ _ __ |  ___|_ _  __ _/ ___|
| | | | '_ \ / _ \ '_ \| |_ / _` |/ _` \___ \
| |_| | |_) |  __/ | | |  _| (_| | (_| |___) |
 \___/| .__/ \___|_| |_|_|  \__,_|\__,_|____/
      |_|


Manage your OpenFaaS functions from the command line"#;

pub fn build_cli() -> App<'static, 'static> {
    let app = App::new("Kubetan")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::VersionlessSubcommands)
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::ColoredHelp)
        .setting(AppSettings::DeriveDisplayOrder)
        .global_settings(&[AppSettings::ColoredHelp])
        .about(WELCOME_MSG);

    let app = Describe::append_subcommand(app);
    let app = Remove::append_subcommand(app);
    let app = NewFunction::append_subcommand(app);
    let app = Up::append_subcommand(app);
    let app = Publish::append_subcommand(app);
    let app = Secret::append_subcommand(app);
    let app = Store::append_subcommand(app);
    let app = Version::append_subcommand(app);
    let app = Cloud::append_subcommand(app);
    let app = Auth::append_subcommand(app);
    let app = RegistryLogin::append_subcommand(app);
    let app = Faas::append_subcommand(app);
    let app = List::append_subcommand(app);
    let app = Login::append_subcommand(app);
    let app = Logout::append_subcommand(app);
    let app = Logs::append_subcommand(app);
    let app = Invoke::append_subcommand(app);
    let app = Namespaces::append_subcommand(app);
    let app = Push::append_subcommand(app);
    let app = Generate::append_subcommand(app);
    let app = Deploy::append_subcommand(app);
    let app = Template::append_subcommand(app);
    let app = Build::append_subcommand(app);
    app
    // .arg(Arg::with_name("region")
    //     .short("r")
    //     .long("region")
    //     .takes_value(true)
    //     .global(true)
    //     .help("Region to use (dev-uk, staging-uk, prod-uk)"))
    // .subcommand(SubCommand::with_name("debug")
    //     .about("Get debug information about a release running in a cluster")
    //     .arg(Arg::with_name("service")
    //         .required(true)
    //         .help("Service name")))
    //
    // .subcommand(SubCommand::with_name("completions")
    //     .about("Generate autocompletion script for Kubetan for the specified shell")
    //     .usage("This can be source using: $ source <(Kubetan completions bash)")
    //     .arg(Arg::with_name("shell")
    //         .required(true)
    //         .possible_values(&Shell::variants())
    //         .help("Shell to generate completions for (zsh or bash)")))
    //
    // .subcommand(SubCommand::with_name("shell")
    //                 .about("Shell into pods for a service described in a manifest")
    //                 .arg(Arg::with_name("service")
    //                     .required(true)
    //                     .help("Service name"))
    //                 .setting(AppSettings::TrailingVarArg)
}
pub async fn run(args: &ArgMatches<'_>) -> Result<()> {
    match dispatch_command(args).await {
        Ok(_) => Err(Error::Custom(
            "command not matched please contact the development team".to_string(),
        )),
        Err(s) => match s {
            State::Matched => Ok(()),
            state => Err(Error::Custom(state.to_string())),
        },
    }
}

pub async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
    Describe::dispatch_command(args).await?;
    Remove::dispatch_command(args).await?;
    NewFunction::dispatch_command(args).await?;
    Up::dispatch_command(args).await?;
    Publish::dispatch_command(args).await?;
    Secret::dispatch_command(args).await?;
    Store::dispatch_command(args).await?;
    Version::dispatch_command(args).await?;
    Cloud::dispatch_command(args).await?;
    Auth::dispatch_command(args).await?;
    RegistryLogin::dispatch_command(args).await?;
    Build::dispatch_command(args).await?;
    List::dispatch_command(args).await?;
    Login::dispatch_command(args).await?;
    Logout::dispatch_command(args).await?;
    Logs::dispatch_command(args).await?;
    Invoke::dispatch_command(args).await?;
    Namespaces::dispatch_command(args).await?;
    Push::dispatch_command(args).await?;
    Generate::dispatch_command(args).await?;
    Deploy::dispatch_command(args).await?;
    Template::dispatch_command(args).await
}
