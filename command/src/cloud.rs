use crate::build::parse_build_args;
use crate::{CommandAppend, State};
use clap::{App, Arg, ArgMatches, SubCommand};
use reqwest::redirect::Attempt;
use reqwest::{Client, Method, Request, StatusCode, Url};
use schema::secret::{KubernetesSecret, KubernetesSecretMetadata};
use std::io::{Read, Write};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use utility::{Error, Result};

pub(crate) struct Cloud;

impl CommandAppend for Cloud {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let seal = SubCommand::with_name("seal")
                .about(r#"faas-cli cloud seal --name alexellis-github --literal hmac-secret=c4488af0c158e8c
  faas-cli cloud seal --name alexellis-token --from-file api-key.txt
  faas-cli cloud seal --name alexellis-token --literal a=b --literal c=d --cert pub-cert.pem
  faas-cli cloud seal --download
  faas-cli cloud seal --download --download-version v0.9.5"#)
                .args_from_usage(
                    "--name            [name]                               'Secret name'
                    -n ,--namespace          [namespace]                      'function name space'
                    -c, --cert               [cert]                           'Filename of public certificate'
                    -o ,--output-file        [output-file]                     'Output file for secrets'
                    --download-version       [download-version]                  'Specify a kubeseal version to download'
                    --download                                                    'Download the kubeseal binary required for this command, see also --download-version'
                    --download-to           [download-to]                           'Specify download path for kubeseal, leave empty for a temp dir'
                    --scope                 [scope]                          'scope for OAuth2 flow - i.e. \"openid profile\"'
                    --grant                 [grant]                          'grant for OAuth2 flow - either implicit, implicit-id or client_credentials'
                    --client-secret         [client-secret]                 'OAuth2 client_secret, for use with client_credentials grant'
             ",
                )
                .arg(
                    Arg::with_name("literal")
                        .help("Secret literal key-value data")
                        .long("literal")
                        //.short("l")
                        .takes_value(true)
                        .multiple(true)
                        .global(true)
                )
                .arg(
                    Arg::with_name("from-file")
                        .help("Read a secret from a from file")
                        .long("from-file")
                        .short("i")
                        .takes_value(true)
                        .multiple(true)
                        .global(true)
                );

        let app = app.subcommand(
            SubCommand::with_name("cloud")
                .about("Commands for operating with OpenFaaS Cloud")
                .subcommand(seal),
        );
        app
    }
}

impl Cloud {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(c_args) = args.subcommand_matches("cloud") {
            if let Some(s_args) = c_args.subcommand_matches("seal") {
                let name = s_args.value_of("name").unwrap_or_default();
                let namespace = s_args.value_of("namespace").unwrap_or("openfaas-fn");
                let cert_file = s_args.value_of("cert").unwrap_or("pub-cert.pem");
                let output_file = s_args.value_of("output-file").unwrap_or("secrets.yml");
                let download = s_args.is_present("download");
                let download_version = s_args.value_of("download-version").unwrap_or_default();
                let download_to = s_args.value_of("download-to").unwrap_or_default();

                let literal: Vec<&str> = s_args.values_of("literal").unwrap_or_default().collect();
                let from_file: Vec<&str> =
                    s_args.values_of("from-file").unwrap_or_default().collect();

                if download {
                    download_kube_seal(download_version, download_to).await
                } else if name.is_empty() {
                    Err(State::Custom("--name is required".to_string()))
                } else {
                    colour::green!("Sealing secret: {} in namespace: {}\n\n", name, namespace);

                    //let enc = base64::

                    //enc: = base64.StdEncoding
                    let mut secret = KubernetesSecret {
                        kind: "v1".to_string(),
                        api_version: "Secret".to_string(),
                        metadata: KubernetesSecretMetadata {
                            name: name.to_string(),
                            namespace: namespace.to_string(),
                        },
                        data: Default::default(),
                    };

                    if !literal.is_empty() {
                        let args = parse_build_args(&literal)?;

                        for (k, v) in args {
                            secret.data.insert(k, v);
                        }
                    }

                    for file in from_file {
                        let out = std::fs::read_to_string(file)
                            .map_err(|e| State::Custom(e.to_string()))?;
                        let path_pars: Vec<&str> = file.split('/').collect();

                        let key = path_pars[path_pars.len() - 1];
                        secret.data.insert(key.to_string(), base64::encode(out));
                    }

                    let sec =
                        serde_json::to_string(&secret).map_err(|e| State::Custom(e.to_string()))?;
                    if std::fs::metadata(cert_file).is_err() {
                        Err(State::Custom(format!(
                            "unable to load public certificate {}",
                            cert_file
                        )))
                    } else {
                        let args = [
                            "--format=yaml".to_string(),
                            "--cert=".to_string() + cert_file,
                        ];
                        let mut kubeseal = std::process::Command::new("kubeseal");

                        kubeseal.args(&args);
                        let process = kubeseal
                            .stdin(Stdio::piped())
                            .stdout(Stdio::piped())
                            .spawn()
                            .map_err(|e| State::Custom(e.to_string()))?;

                        match process.stdin {
                            None => Err(State::Custom("can't have pip".to_string())),
                            Some(mut stdin) => {
                                stdin
                                    .write_all(sec.as_bytes())
                                    .map_err(|e| State::Custom(e.to_string()))?;

                                match process.stdout {
                                    None => Err(State::Custom(
                                        "unable to start \"kubeseal\", check it is installed"
                                            .to_string(),
                                    )),
                                    Some(mut stdout) => {
                                        let mut out = String::new();
                                        stdout
                                            .read_to_string(&mut out)
                                            .map_err(|e| State::Custom(e.to_string()))?;

                                        //todo check permission
                                        //writeErr := ioutil.WriteFile(outputFile, out, 0755)
                                        std::fs::write(output_file, out).map_err(|e| {
                                            State::Custom(format!(
                                                "unable to write secret: to {} and error: {}",
                                                output_file, e
                                            ))
                                        })?;

                                        colour::green!("{} written.\n", output_file);
                                        Err(State::Matched)
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                let usage = c_args.usage();

                //println!("{}");

                Err(State::Custom(format!(
                    "cloud command must followed by sub command\n\
                {}\n\
  faas-cli cloud seal --name alexellis-github --literal hmac-secret=c4488af0c158e8c
  faas-cli cloud seal --name alexellis-token --from-file api-key.txt
  faas-cli cloud seal --name alexellis-token --literal a=b --literal c=d --cert pub-cert.pem
  faas-cli cloud seal --download
  faas-cli cloud seal --download --download-version v0.9.5
                ",
                    usage
                )))
            }
        } else {
            Ok(())
        }
    }
}

async fn download_kube_seal(download_version: &str, download_to: &str) -> crate::Result {
    let releases = "https://github.com/bitnami-labs/sealed-secrets/releases/latest";

    let release_version = if download_version.is_empty() {
        find_release(releases).await?
    } else {
        download_version.to_string()
    };

    let os = std::env::consts::OS;
    let mut arch = std::env::consts::ARCH;

    if arch == "x86_64" {
        arch = "amd64"
    }

    let download_url = "https://github.com/bitnami/sealed-secrets/releases/download/".to_string()
        + release_version.as_str()
        + "/kubeseal-"
        + os
        + "-"
        + arch;

    colour::blue!(
        "Starting download of kubeseal {}, this could take a few moments.\n",
        release_version
    );
    let output = download_binary(
        Client::new(),
        download_url.as_str(),
        "kubeseal",
        download_to,
    )
    .await?;

    colour::blue!(
        r#"Download completed, please run:

        chmod +x {0}
        {0} --version
               sudo install {0} /usr/local/bin/

               "#,
        output
    );

    Ok(())
}

pub(crate) async fn find_release(url: &str) -> Result<String> {
    let policy = |a: Attempt| a.error(Error::Custom(format!("net/http: use last response")));
    let policy = reqwest::redirect::Policy::custom(policy);
    let client = reqwest::ClientBuilder::new()
        .timeout(Duration::from_secs(5))
        .redirect(policy)
        .build()
        .map_err(|e| Error::Custom(e.to_string()))?;
    let uri = reqwest::Url::parse(url).map_err(|e| Error::Custom(e.to_string()))?;
    let req = client
        .request(Method::HEAD, uri)
        .build()
        .map_err(|e| Error::Custom(e.to_string()))?;

    let res = client
        .execute(req)
        .await
        .map_err(|e| Error::Custom(e.to_string()))?;

    let status_code = res.status();
    if status_code.as_u16() != 302 {
        return Err(Error::Custom(format!(
            "incorrect status code: {}",
            status_code
        )));
    }

    if let Some(loc) = res.headers().get("location") {
        let location = loc.to_str().map_err(|e| Error::Custom(e.to_string()))?;
        let index = location.rfind('/').map(|idx| idx as isize).unwrap_or(-1) + 1;
        let (_, version) = location.split_at(index as usize);
        Ok(version.to_string())
    } else {
        Err(Error::Custom(
            "unable to determine release of kubeseal".to_string(),
        ))
    }
}

async fn download_binary(
    client: Client,
    url: &str,
    name: &str,
    download_to: &str,
) -> Result<String> {
    let uri = Url::parse(url).map_err(|e| Error::Custom(e.to_string()))?;

    let req = Request::new(Method::GET, uri);
    let res = client
        .execute(req)
        .await
        .map_err(|e| Error::Custom(e.to_string()))?;
    let status_code = res.status();

    if status_code != StatusCode::OK {
        return Err(Error::Custom(format!("could not find release, http status code was {}, release may not exist for this architecture"
                                 , status_code)));
    }

    let mut temp_dir = if download_to.is_empty() {
        std::env::temp_dir()
    } else {
        Path::new(download_to).to_owned()
    };

    temp_dir.push(name);
    let output_path = temp_dir.to_string_lossy().to_string();
    let body = res.text().await.map_err(|e| Error::Custom(e.to_string()))?;

    if !body.is_empty() {
        //todo check permission
        //err := ioutil.WriteFile(outputPath, res, 0600)
        std::fs::write(output_path.as_str(), body)?;

        Ok(output_path)
    } else {
        Err(Error::Custom(format!("error downloading {}", url)))
    }
}
