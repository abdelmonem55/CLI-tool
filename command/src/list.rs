use crate::faas::{check_and_set_default_yaml, DEFAULT_GATEWAY};
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::{CommandAppend, State};
use clap::{App, Arg, ArgMatches, SubCommand};
use proxy::auth::ClientAuthE;
use proxy::auth::ClientAuthE::{BasicAuth, BearerToken};
use proxy::client::Client;
use stack::stack::parse_yaml_file;

pub(crate) struct List;

impl CommandAppend for List {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("list")
                .about("Lists OpenFaaS functions either on a local or remote gateway")
                // .arg_from_usage("<name> 'function name'")
                //.arg_from_usage("-g, --gateway [gateway]")
                .arg_from_usage("-o ,--output [output=text/json]")
                .args_from_usage(
                    "-v ,--verbose 'Verbose output for the function list'\n
            --tls-no-verify 'Disable TLS validation'
            -t ,--token   'Pass a JWT token to use instead of basic auth'
            -q ,--quiet              'Quiet mode - print out only the function's ID'
            ",
                )
                .arg(
                    Arg::with_name("sort")
                        .long("sort")
                        .default_value("name")
                        .takes_value(true)
                        .global(true)
                        .help(r#"Sort the functions by "name" or "invocations"#),
                )
                .arg(
                    Arg::with_name("namespace")
                        .long("namespace")
                        .short("n")
                        .default_value("")
                        .takes_value(true)
                        .global(true)
                        .help("Namespace of the function"),
                ),
        );
        app
    }
}

impl List {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        let res = if let Some(largs) = args.subcommand_matches("list") {
            let regex = args.value_of("regex").unwrap_or("");
            let filter = args.value_of("filter").unwrap_or("");
            let gateway = args.value_of("gateway").ok_or(State::Custom(format!(
                "you must set gateway using \
             --gateway, -g http://host"
            )))?;
            let envsubst = true; //args.is_present("envsubst");

            let yaml = args
                .value_of("yaml")
                .unwrap_or(check_and_set_default_yaml().unwrap_or_default());
            let mut service = Default::default();
            if !yaml.is_empty() {
                service = parse_yaml_file(yaml, regex, filter, envsubst)
                    .await
                    .map_err(|e| State::Error(utility::Error::Custom(format!("{}", e))))?;
            }
            let token = largs.value_of("token").unwrap_or("").to_string();
            let namespace = largs.value_of("namespace").unwrap_or("").to_string();
            let sort_by = largs.value_of("sort").unwrap_or("name").to_string();
            let quiet = largs.is_present("quiet");
            let verbose = largs.is_present("verbose");

            let env_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or("".into());
            let gateway_address = get_gateway_url(
                gateway,
                DEFAULT_GATEWAY,
                service.provider.gateway_url.as_str(),
                env_url.as_str(),
            );

            let cli_auth = ClientAuthE::new(token.as_str(), gateway_address.as_str())
                .map_err(|e| State::Error(e))?;

            // let transport = get_de
            // transport := GetDefaultCLITransport(tlsInsecure, &commandTimeout)
            let basic;
            let bearer;
            let mut proxy_client = match cli_auth {
                BasicAuth(b) => {
                    basic = b;
                    Client::new(Box::new(&basic), gateway)
                }
                BearerToken(b) => {
                    bearer = b;
                    Client::new(Box::new(&bearer), gateway)
                }
            }
            .map_err(|e| State::Error(e))?;

            let mut functions = proxy_client
                .list_functions(namespace.as_str())
                .await
                .map_err(|e| State::Error(e))?;

            if sort_by == "name" {
                functions.sort_by(|a, b| a.name.cmp(&b.name));
            } else if sort_by == "invocations" {
                functions
                    .sort_by(|a, b| a.invocation_count.partial_cmp(&b.invocation_count).unwrap());
            } else if sort_by == "creation" {
                //todo check if created_at valid date before unwrap it
                functions.sort_by(|a, b| {
                    let t1 = chrono::DateTime::parse_from_rfc3339(&a.created_at).unwrap();
                    let t2 = chrono::DateTime::parse_from_rfc3339(&b.created_at).unwrap();
                    t1.cmp(&t2)
                });
            }

            if quiet {
                for func in &functions {
                    println!("{}\n", func.name)
                }
            } else if verbose {
                let mut max = 40;
                for func in &functions {
                    let len = func.image.len();
                    if len > max {
                        max = len;
                    }
                }
                println!(
                    "{}",
                    fun_format_with_space(
                        "Function",
                        "Image",
                        "Invocations",
                        "Replicas",
                        "CreatedAt",
                        max
                    )
                );
                for function in &functions {
                    let str = fun_format_with_space(
                        function.name.as_str(),
                        function.image.as_str(),
                        function.invocation_count.to_string().as_str(),
                        function.replicas.to_string().as_str(),
                        function.created_at.as_str(),
                        max,
                    );
                    println!("{}", str);
                }
            } else {
                println!(
                    "{}",
                    format_with_space("Function", "Invocations", "Replicas")
                );
                for function in &functions {
                    let str = format_with_space(
                        function.name.as_str(),
                        function.invocation_count.to_string().as_str(),
                        function.replicas.to_string().as_str(),
                    );
                    println!("{}", str);
                }
            }
            //return error in match to easy use ? to check next subcommand
            Err(State::Matched)
        } else {
            Ok(())
        };
        res
    }
}

// fn sort_func_status_by_data(date1:&str,date2:&str)->utility::Result<Ordering>{
//     let date1 = chrono::DateTime::parse_from_rfc3339(date1)
//         .map_err(|e| utility::Error::IoCustom(format!("{:?}",e)))?;
//     let date2 = chrono::DateTime::parse_from_rfc3339(date2)
//         .map_err(|e| utility::Error::IoCustom(format!("{:?}",e)))?;
//    Ok(date1.cmp(&date2))
// }

fn fun_format_with_space(
    str1: &str,
    str2: &str,
    str3: &str,
    str4: &str,
    str5: &str,
    width: usize,
) -> String {
    // let format =format!("{:width$}", "Function", width=30) + format!("{:width$}", "Image", width=max).as_str()
    //     +format!("{:width$}", "Invocations", width=15).as_str() +format!("{:width$}", "Replicas", width=5).as_str()
    //     +format!("{:width$}", "CreatedAt", width=5).as_str();

    let format = format!("{:width$}", str1, width = 30)
        + format!("{:width$}", str2, width = width).as_str()
        + format!("{:width$}", str3, width = 15).as_str()
        + format!("{:width$}", str4, width = 15).as_str()
        + format!("{:width$}", str5, width = 15).as_str();
    format
}

fn format_with_space(str1: &str, str2: &str, str3: &str) -> String {
    let format = format!("{:width$}", str1, width = 30)
        + format!("{:width$}", str2, width = 15).as_str()
        + format!("{:width$}", str3, width = 5).as_str();
    format
}
