use crate::error::check_tls_insecure;
use crate::faas::DEFAULT_GATEWAY;
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::{State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};
use proxy::auth::ClientAuthE;
use std::io::Read;
use utility::faas::types::model::Secret;
use utility::{Error, Result};

pub(crate) struct SecretCreate;

impl SubCommandAppend for SecretCreate {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app =
            SubCommand::with_name("create")
                .about(r#"The create command creates a new secret from file, literal or STDIN`,
	Example: `faas-cli secret create secret-name --from-literal=secret-value
faas-cli secret create secret-name --from-literal=secret-value --gateway=http://127.0.0.1:8080
faas-cli secret create secret-name --from-file=/path/to/secret/file --gateway=http://127.0.0.1:8080
cat /path/to/secret/file | faas-cli secret create secret-name"#)
                .arg(Arg::with_name("SECRET-NAME")
                    .index(1)
                    .required(true)
                    .help("secret name")
                )
                //.arg(Arg::with_name("STDIN")
                //     .index(2)
                //     .help("secret from stdin")
                // )
                .args_from_usage("
                   --from-literal [from-literal ]                      'Value of the secret'
                   --from-file  [from-file]             'Path to the secret file'
                   --trim                               'trim whitespace from the start and end of the secret value'
                   --tls-no-verify                      'Disable TLS validation'
                   -k, --token [token]                      'Pass a JWT token to use instead of basic auth'
                   -n, --namespace  [namespace]             'Namespace of the function'
                ");

        app
    }
}

impl SecretCreate {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(c_args) = args.subcommand_matches("create") {
            let secret_name = c_args
                .value_of("SECRET-NAME")
                .ok_or(State::Custom("secret name is required".to_string()))?;
            let secret_file = c_args.value_of("from-file").unwrap_or_default();
            let function_namespace = c_args.value_of("namespace").unwrap_or_default();
            let gateway = args.value_of("gateway").unwrap_or(DEFAULT_GATEWAY);
            let token = c_args.value_of("token").unwrap_or_default();
            let tls_no_verify = c_args.is_present("tls-no-verify");

            let trim_secret = true; //c_args.is_present("trim");
                                    //let from_stdin = c_args.is_present("from-stdin")
            let literal_secret = c_args.value_of("from-secret").unwrap_or_default();

            // let secrets =[!secret_file.is_empty(),!secret_stdin.is_empty(),!literal_secret.is_empty()];
            //
            // if secrets.iter().filter(|e| e == true).collect::<[bool]>().len() > 1{
            //
            // }

            if !secret_file.is_empty() && !literal_secret.is_empty() {
                //|| (!secret_file.is_empty() && !from_stdin.is_empty())|| (!from_stdin.is_empty() && !literal_secret.is_empty()){
                return Err(State::Custom("please provide secret using only one option from [--from-literal secret], [--from-file secret] and STDIN (catch input data)".to_string()));
            }

            validate_secret_name(secret_name)?;

            let mut secret = Secret {
                name: secret_name.to_string(),
                namespace: function_namespace.to_string(),
                value: "".to_string(),
            };

            if !literal_secret.is_empty() {
                secret.value = literal_secret.to_string();
            } else if !secret_file.is_empty() {
                secret.value = read_secret_from_file(secret_file)?;
            } else {
                let mut secret_stdin = Vec::new();
                std::io::stdin()
                    .read_to_end(&mut secret_stdin)
                    .map_err(|e| State::Custom(e.to_string()))?;
                secret.value = std::str::from_utf8(secret_stdin.as_slice())
                    .map_err(|e| State::Custom(e.to_string()))?
                    .to_string();
            }

            if trim_secret {
                //secret.Value = strings.TrimSpace(secret.Value)
                secret.value = secret.value.trim().to_string();
            }

            if !secret.value.is_empty() {
                return Err(State::Custom(
                    "must provide a non empty secret via --from-literal, --from-file or STDIN"
                        .to_string(),
                ));
            }

            let openfaas_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or_default();
            let gateway_address =
                get_gateway_url(gateway, DEFAULT_GATEWAY, "", openfaas_url.as_str());

            let msg = check_tls_insecure(gateway_address.as_str(), tls_no_verify);
            if !msg.is_empty() {
                colour::yellow!("{}\n", msg);
            }
            let client_auth = ClientAuthE::new(token, gateway_address.as_str())?;
            //transport := GetDefaultCLITransport(tlsInsecure, &commandTimeout)
            let client = client_auth.get_client(gateway_address.as_str())?;

            colour::green!("Creating secret: {}", secret.name);
            let (_, output) = client.create_secret(&secret).await?;
            colour::green!("{}", output);

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

/// Kubernetes DNS-1123 Subdomain Regex
/// https://github.com/kubernetes/kubernetes/blob/6902f3112d98eb6bd0894886ff9cd3fbd03a7f79/staging/src/k8s.io/apimachinery/pkg/util/validation/validation.go#L131

//const DNS1123_LABEL_FMT          :&str = "[a-z0-9]([-a-z0-9]*[a-z0-9])?";
const DNS1123_SUBDOMAIN_FMT: &str =
    "[a-z0-9]([-a-z0-9]*[a-z0-9])?(\\.[a-z0-9]([-a-z0-9]*[a-z0-9])?)*"; //DNS1123LABELFMT + "(\\." + DNS1123LABELFMT + ")*";
const INVALID_SECRET_NAME_MESSAGE:&str = "ERROR: invalid secret name %s\nSecret name must start and end with an alphanumeric character \nand can only contain lower-case alphanumeric characters, '-' or '.'";

pub(crate) fn validate_secret_name(secret_name: &str) -> Result<()> {
    let regex_fmt = format!("^{}$", DNS1123_SUBDOMAIN_FMT);
    let dns1123_subdomain_regex = regex::Regex::new(regex_fmt.as_str())
        .map_err(|e| Error::Custom(format!("invalid regex for {} error {}", regex_fmt, e)))?;

    if dns1123_subdomain_regex.is_match(secret_name) {
        Ok(())
    } else {
        Err(Error::Custom(format!(
            "{} ,{}",
            INVALID_SECRET_NAME_MESSAGE, secret_name
        )))
    }
}

pub(crate) fn read_secret_from_file(secret_file: &str) -> Result<String> {
    let data = std::fs::read_to_string(secret_file)?;
    Ok(data)
}
