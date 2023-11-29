use crate::{CommandAppend, State};
use clap::{App, ArgMatches, SubCommand};
use std::collections::HashMap;
use std::io::{ErrorKind, Read};
use utility::{Error, Result};

pub(crate) struct RegistryLogin;

impl CommandAppend for RegistryLogin {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("registry-login")
                .about(r#"Generate and save the registry authentication file"#)
                .args_from_usage(
                    "-u ,--username [username]      'The Registry Username'
                    --server [server]                       'The server URL, it is defaulted to the docker registry'
                    -p ,--password [password]               'The Registry password'
                    -s, --password-stdin                         'Reads the docker password from stdin, either pipe to the command or remember to press ctrl+d when reading interactively'
                    --ecr                                       'If we are using ECR we need a different set of flags, so if this is set, we need to set --username and --password'
                    --account-id    [account-id]                'Your AWS Account id'
                    --region [region]                           'Your AWS region'
             ",
                )
        );
        app
    }
}

impl RegistryLogin {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(l_args) = args.subcommand_matches("registry-login") {
            let username = l_args.value_of("username").ok_or(State::Custom(format!(
                "you must set username using \
             --username VALUE (-u)"
            )))?;
            let account_id = l_args.value_of("account-id").unwrap_or_default();
            let region = l_args.value_of("region").unwrap_or_default();
            let server = l_args
                .value_of("server")
                .unwrap_or("https://index.docker.io/v1/");
            let ecr_enabled = l_args.is_present("ecr");

            if ecr_enabled {
                generate_ecr_file(account_id, region)?;
            } else {
                let password_stdin_presented = l_args.is_present("password-stdin");
                let mut password = String::new();

                if let Some(pass) = args.value_of("password") {
                    println!("WARNING! Using --password is insecure, consider using: cat ~/faas_pass.txt | faas-cli login -u user --password-stdin");
                    if password_stdin_presented {
                        return Err(State::Custom(
                            "--password and --password-stdin are mutually exclusive".into(),
                        ));
                    }
                    password = pass.to_string();
                }

                if password_stdin_presented {
                    let mut password_stdin = Vec::new();
                    std::io::stdin()
                        .read_to_end(&mut password_stdin)
                        .map_err(|e| State::Custom(e.to_string()))?;
                    let pass = std::str::from_utf8(password_stdin.as_slice())
                        .map_err(|e| State::Custom(e.to_string()))?;
                    password = pass.to_string();

                    //password = strings.TrimSpace(string(passwordStdin))
                }
                password = password.trim().to_string();
                if password.is_empty() {
                    return Err(State::Custom(
                        "must provide a non-empty password via --password or --password-stdin"
                            .to_string(),
                    ));
                }

                generate_file(username, password.as_str(), server)?;
            }

            colour::green!("\nWrote ./credentials/config.json..OK\n");
            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

fn generate_file(username: &str, password: &str, server: &str) -> Result<()> {
    let data = generate_registry_auth(server, username, password)?;
    write_file_to_fass_cli_tmp(data.as_str())
}

fn generate_ecr_file(account_id: &str, region: &str) -> Result<()> {
    let data = generate_ecr_registry_auth(account_id, region)?;
    write_file_to_fass_cli_tmp(data.as_str())
}

fn generate_registry_auth(server: &str, username: &str, password: &str) -> Result<String> {
    if username.is_empty() || password.is_empty() || server.is_empty() {
        return Err(Error::Custom(
            "both --username and (--password-stdin or --password) are required and server"
                .to_string(),
        ));
    }

    let encoded_string = base64::encode(&format!("{}:{}", username, password));
    let auth = Auth {
        base64_auth_string: encoded_string,
    };
    let mut auth_configs = HashMap::new();
    auth_configs.insert(server.to_string(), auth);
    let data = RegistryAuth { auth_configs };

    serde_json::to_string(&data).map_err(|e| Error::Custom(e.to_string()))
}

fn generate_ecr_registry_auth(account_id: &str, region: &str) -> Result<String> {
    if account_id.is_empty() || region.is_empty() {
        return Err(Error::Custom(
            "you must provide an --account-id and --region when using --ecr".to_string(),
        ));
    }
    let mut cred_helpers = HashMap::new();
    cred_helpers.insert(
        format!("{}.dkr.ecr.{}.amazonaws.com", account_id, region),
        "ecr-login".to_string(),
    );
    let data = ECRRegistryAuth {
        creds_store: "ecr-login".to_string(),
        cred_helpers,
    };

    serde_json::to_string(&data).map_err(|e| Error::Custom(e.to_string()))
}

fn write_file_to_fass_cli_tmp(file_data: &str) -> Result<()> {
    let path = "./credentials";
    if let Err(e) = std::fs::metadata(path) {
        if e.kind() == ErrorKind::NotFound {
            #[cfg(target_os = "windows")]
            {
                std::fs::create_dir(path).map_err(|e| {
                    Error::Custom(format!("Error creating path: {} - {}.\n", path, e))
                })?;
            }

            #[cfg(target_os = "unix")]
            {
                use std::fs::DirBuilder;
                use std::os::unix::fs::DirBuilderExt;
                let permission = DEFAULT_DIR_PERMISSION.load(Ordering::Relaxed);
                let mut builder = DirBuilder::new();
                builder.mode(0744).create(path).map_err(|e| {
                    Error::Custom(format!("Error creating path: {} - {}.\n", path, e))
                })?;
            }
        } else {
            return Err(Error::Io(e));
        }
    }

    std::fs::write(format!("{}/config.json", path), file_data)?;

    Ok(())
}

#[derive(serde::Serialize)]
struct Auth {
    #[serde(rename = "auth")]
    pub base64_auth_string: String, //`json:"auth"`
}

#[derive(serde::Serialize)]
struct RegistryAuth {
    #[serde(rename = "auths")]
    pub auth_configs: HashMap<String, Auth>, //`json:"auths"`
}

#[derive(serde::Serialize)]
struct ECRRegistryAuth {
    #[serde(rename = "credsStore")]
    pub creds_store: String, //`json:"credsStore"`
    #[serde(rename = "credHelpers")]
    pub cred_helpers: HashMap<String, String>, //`json:"credHelpers"`
}
