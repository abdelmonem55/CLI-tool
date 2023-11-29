#![allow(dead_code)]
use crate::CommandAppend;
use clap::{App, Arg};

pub const DEFAULT_GATEWAY: &str = "http://127.0.0.1:8080";
pub const DEFAULT_NETWORK: &str = "";
pub const DEFAULT_YAML: &str = "stack.yml";
pub const DEFAULT_SCHEMA_VERSION: &str = "1.0";

pub(crate) struct Faas;

impl CommandAppend for Faas {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app
            .arg(
                Arg::with_name("yaml")
                    .long("yaml")
                    .short("f")
                    .global(true)
                    .takes_value(true)
                    //.default_value()
                    .help("Path to YAML file describing function(s)"),
            )
            .arg(
                Arg::with_name("regex")
                    .long("regex")
                    .takes_value(true)
                    .global(true)
                    .help("Regex to match with function names in YAML file"),
            )
            .arg(
                Arg::with_name("filter")
                    .long("filter")
                    .takes_value(true)
                    .global(true)
                    .help("Wildcard to match with function names in YAML file"),
            )
            .arg(
                Arg::with_name("gateway")
                    .long("gateway")
                    .short("g")
                    .default_value(DEFAULT_GATEWAY)
                    .takes_value(true)
                    .global(true)
                    .help("Gateway URL starting with http(s)://"),
            )
            .arg(
                Arg::with_name("envsubst")
                    .long("envsubst")
                    .global(true)
                    .help("Substitute environment variables in stack.yml file"),
            );
        app
    }
}

// impl Faas {
//     #[inline(always)]
//     fn dispatch_command<'s>(args: &ArgMatches<'s>) -> crate::Result {
//         if let Some(a) = args.subcommand_matches("list-services") {}
//         Ok(())
//     }
// }

// Flags that are to be added to all commands.
// var (
// yamlFile string
// regex    string
// filter   string
// )

// Flags that are to be added to subset of commands.
// var (
// fprocess     string
// functionName string
// handlerDir   string
// network      string
// gateway      string
// handler      string
// image        string
// imagePrefix  string
// language     string
// tlsInsecure  bool
// )

pub(crate) fn check_and_set_default_yaml() -> utility::Result<&'static str> {
    // Check if there is a default yaml file and set it

    std::fs::metadata(DEFAULT_YAML)
        .map(|_| DEFAULT_YAML)
        .map_err(|e| utility::Error::Io(e))
}
