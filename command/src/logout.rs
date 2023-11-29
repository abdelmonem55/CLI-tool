use crate::faas::DEFAULT_GATEWAY;
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::{CommandAppend, State};
use clap::{App, ArgMatches, SubCommand};
use config::config_file::remove_auth_config;

pub(crate) struct Logout;

impl CommandAppend for Logout {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("logout")
                .about(r#"Log out from OpenFaaS gateway.\nIf no gateway is specified, the default local one will be used.",
	Example: `  faas-cli logout --gateway https://openfaas.mydomain.com`"#)
        );
        app
    }
}

impl Logout {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(_) = args.subcommand_matches("logout") {
            let gateway = args.value_of("gateway").ok_or(State::Custom(format!(
                "you must set gateway using \
             --gateway, -g http://host"
            )))?;

            let gateway = gateway.trim();
            let openfass_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or("".into());
            let gateway = get_gateway_url(gateway, DEFAULT_GATEWAY, "", openfass_url.as_str());
            remove_auth_config(gateway.as_str())?;
            println!("credentials removed for {}", gateway);
            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}
