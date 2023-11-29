use crate::cmds::{build_cli, run};
use clap::{App, ArgMatches};

use utility::Error;

pub(crate) mod build;
pub mod cmds;
pub(crate) mod deploy;
pub(crate) mod error;
pub(crate) mod faas;
pub(crate) mod fetch_template;
pub(crate) mod generate;
pub(crate) mod invoke;
pub(crate) mod list;
pub(crate) mod login;
pub(crate) mod logout;
pub(crate) mod logs;
pub mod logs_format;
pub(crate) mod namespaces;
pub(crate) mod priority;
pub(crate) mod push;
pub(crate) mod template_store;
pub(crate) mod validate;
//command
pub(crate) mod template;
//subcommands
pub(crate) mod template_pull;
pub(crate) mod template_pull_stack;

pub(crate) mod auth;
pub(crate) mod cloud;
pub(crate) mod registry_login;
pub(crate) mod version;

pub(crate) mod template_store_describe;
pub(crate) mod template_store_list;
pub(crate) mod template_store_pull;

pub(crate) mod describe;
pub(crate) mod new_function;
pub(crate) mod publish;
pub(crate) mod remove;
pub(crate) mod secret;
pub(crate) mod secret_create;
mod secret_list;
pub(crate) mod secret_remove;
pub(crate) mod secret_update;
pub(crate) mod store;
pub(crate) mod store_deploy;
pub(crate) mod store_describe;
pub(crate) mod store_list;
pub(crate) mod up;
pub(crate) mod update_gitignore;

type Result = std::result::Result<(), State>;
// pub(crate) enum State{
//     PassNext,
//
// }

#[derive(thiserror::Error, Debug)]
pub enum State {
    #[error("command matched")]
    Matched,
    #[error("can't catch the subs command")]
    Unreachable,
    #[error("{0}")]
    Custom(String),
    #[error("{0}")]
    Error(#[from] Error),
}

//return OK(()) , or Err(State::Error(e))
pub fn extract_error(res: crate::Result) -> std::result::Result<(), State> {
    if let Err(state) = res {
        if let State::Error(error) = state {
            return Err(State::Error(error));
        }
    }
    Ok(())
}
// impl ToString for State{
//     fn to_string(&self) -> String {
//         match self{
//             State::Matched => {
//                 String::from("command matched")
//             }
//             State::Unreachable => {
//                 String::from("unreachable code")
//             }
//             State::Error(e) => {
//                 format!("{}",e)
//             }
//             State::Custom(str)=> {str.to_owned()}
//         }
//     }
// }
pub async fn exec() {
    // completions handling first

    // let name = args.subcommand_name().unwrap();
    let app = build_cli();
    let args: ArgMatches = app.get_matches();
    if let Err(err) = run(&args).await {
        println!("{}", err.to_string());
    }
}
pub(crate) trait CommandAppend {
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static>;
}
pub(crate) trait SubCommandAppend {
    fn append_subcommand() -> App<'static, 'static>;
}

use async_trait::async_trait;

#[async_trait]
pub trait CommandDispatch<'s> {
    async fn dispatch_command(args: &ArgMatches<'s>) -> crate::Result;
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
