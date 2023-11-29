use crate::cloud::find_release;
use crate::faas::{check_and_set_default_yaml, DEFAULT_GATEWAY};
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::{CommandAppend, State};
use clap::{App, ArgMatches, SubCommand};
use proxy::auth::ClientAuthE;
use stack::stack::parse_yaml_file;
use utility::Result;
use version::version::{build_version, get_git_commit, get_version};

pub(crate) struct Version;

impl CommandAppend for Version {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("version")
                .about(r#"The version command returns the current clients version information.

This currently consists of the GitSHA from which the client was built.
- https://github.com/openfaas/faas-cli/tree/{},
	Example: `  faas-cli version
  faas-cli version --short-version"#)
                .args_from_usage("
                    --short-version                                      'Just print Git SHA'
                    --tls-no-verify                                     'Disable TLS validation'
                    --warn-update                                             'Check for new version and warn about updating'
                    -k, --token          [token]                          'Pass a JWT token to use instead of basic auth'

             ",
                )
        );
        app
    }
}

impl Version {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(v_args) = args.subcommand_matches("version") {
            let short_version = v_args.is_present("short-version");
            let warn_update = true; //v_args.is_present("warn-update");
                                    // let tls_no_verify  = v_args.is_present("tls-no-verify ");
            let token = v_args.value_of("token").unwrap_or("");

            let gateway = args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
            let filter = args.value_of("filter").unwrap_or_default();
            let regex = args.value_of("regex").unwrap_or_default();
            let envsubst = true; //args.is_present("envsubst");
            let yaml_file = args
                .value_of("yaml")
                .unwrap_or(check_and_set_default_yaml().unwrap_or_default());

            let releases = "https://github.com/openfaas/faas-cli/releases/latest";

            if short_version {
                colour::yellow!("{}", build_version()?)
            } else {
                print_logo();

                println!(
                    r#"CLI:
                    commit:  {}
                           version: {}
                           "#,
                    get_git_commit()?,
                    build_version()?
                );
                print_server_versions(gateway, yaml_file, token, regex, filter, envsubst).await?;
            }

            if warn_update {
                //todo check version maybe never set
                let version = get_version()?;
                let latest = find_release(releases).await.map_err(|e| {
                    State::Custom(format!("unable to find latest version online error: {}", e))
                })?;

                if !version.is_empty() && version != latest {
                    colour::yellow!("Your faas-cli version ({}) may be out of date. Version: {} is now available on GitHub.\n"
                        , version, latest)
                }
            }

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

async fn print_server_versions(
    gateway: &str,
    yaml_file: &str,
    token: &str,
    regex: &str,
    filter: &str,
    envsubst: bool,
) -> Result<()> {
    let services = if !yaml_file.is_empty() {
        parse_yaml_file(yaml_file, regex, filter, envsubst)
            .await
            .unwrap_or_default()
    } else {
        Default::default()
    };
    let openfaas_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or_default();
    let gateway_address = get_gateway_url(
        gateway,
        DEFAULT_GATEWAY,
        services.provider.gateway_url.as_str(),
        openfaas_url.as_str(),
    );

    let cli_auth = ClientAuthE::new(token, gateway_address.as_str())?;

    // versionTimeout := 5 * time.Second
    // transport := GetDefaultCLITransport(tlsInsecure, &versionTimeout)

    let client = cli_auth.get_client(gateway_address.as_str())?;
    let gateway_info = client.get_system_info().await?;

    print_gateway_details(
        gateway_address.as_str(),
        gateway_info.version.release.as_str(),
        gateway_info.version.sha.as_str(),
    );

    colour::blue!(
        "
               Provider
               name:          {}
               orchestration: {}
               version:       {}
               sha:           {}
               ",
        gateway_info.provider.name,
        gateway_info.provider.orchestration,
        gateway_info.provider.version.release,
        gateway_info.provider.version.sha
    );
    println!();
    Ok(())
}

fn print_gateway_details(gateway_address: &str, version: &str, sha: &str) {
    colour::blue!(
        "
               Gateway
               uri:     {}",
        gateway_address
    );

    if !version.is_empty() {
        colour::blue!(
            "
                   version: {}
                   sha:     {}
                   ",
            version,
            sha
        );
    }

    println!();
}

/// printLogo prints an ASCII logo, which was generated with figlet
pub(crate) fn print_logo() {
    // figletColoured := aec.BlueF.Apply(figletStr)
    // if runtime.GOOS == "windows" {
    //     figletColoured = aec.GreenF.Apply(figletStr)
    // }
    if std::env::consts::OS == "windows" {
        colour::green!("{}", FIGLETSTR);
    } else {
        colour::blue!("{}", FIGLETSTR);
    }
}

const FIGLETSTR: &str = r#"  ___                   _____           ____
 / _ \ _ __   ___ _ __ |  ___|_ _  __ _/ ___|
| | | | '_ \ / _ \ '_ \| |_ / _` + "`" + ` |/ _` + "`" + ` \___ \
| |_| | |_) |  __/ | | |  _| (_| | (_| |___) |
 \___/| .__/ \___|_| |_|_|  \__,_|\__,_|____/
      |_|
"#;
