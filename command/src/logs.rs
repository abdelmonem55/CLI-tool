use crate::error::check_tls_insecure;
use crate::faas::DEFAULT_GATEWAY;
use crate::logs_format::get_log_formatter;
use crate::priority::{get_gateway_url, OPENFAAS_URL_ENVIRONMENT};
use crate::{CommandAppend, State};
use chrono::DateTime;
use clap::{App, Arg, ArgMatches, SubCommand};
use proxy::auth::ClientAuthE;
use std::time::Duration;
use utility::faas_provider::logs::Request;

pub(crate) struct Logs;

// cmd.Flags().StringVarP(&gateway, "gateway", "g", defaultGateway, "Gateway URL starting with http(s)://")
// cmd.Flags().StringVarP(&functionNamespace, "namespace", "n", "", "Namespace of the function")
//
// cmd.Flags().BoolVar(&tlsInsecure, "tls-no-verify", false, "Disable TLS validation")
//
// cmd.Flags().DurationVar(&logFlagValues.since, "since", 0*time.Second, "return logs newer than a relative duration like 5s")
// cmd.Flags().Var(&logFlagValues.sinceTime, "since-time", "include logs since the given timestamp (RFC3339)")
// cmd.Flags().IntVar(&logFlagValues.lines, "lines", -1, "number of recent log lines file to display. Defaults to -1, unlimited if <=0")
// cmd.Flags().BoolVarP(&logFlagValues.tail, "tail", "t", true, "tail logs and continue printing new logs until the end of the request, up to 30s")
// cmd.Flags().StringVarP(&logFlagValues.token, "token", "k", "", "Pass a JWT token to use instead of basic auth")
//
// logFlagValues.timeFormat = flags.TimeFormat(time.RFC3339)
// cmd.Flags().VarP(&logFlagValues.logFormat, "output", "o", "output logs as (plain|keyvalue|json), JSON includes all available keys")
// cmd.Flags().Var(&logFlagValues.timeFormat, "time-format", "string format for the timestamp, any value go time format string is allowed, empty will not print the timestamp")
// cmd.Flags().BoolVar(&logFlagValues.includeName, "name", false, "print the function name")
// cmd.Flags().BoolVar(&logFlagValues.includeInstance, "instance", false, "print the function instance name/id")
impl CommandAppend for Logs {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("logs")
                .about(r#"Fetch logs for a given function name in plain text or JSON format.",
	Example: `  faas-cli logs FN
  faas-cli logs FN --output=json
  faas-cli logs FN --lines=5
  faas-cli logs FN --tail=false --since=10m
  faas-cli logs FN --tail=false --since=2010-01-01T00:00:00Z"#)
                // .arg_from_usage("<name> 'function name'")
                //.arg_from_usage("-g, --gateway [gateway]")

                .arg(
                    Arg::with_name("tail")
                        .long("tail")
                        .short("t")
                        .global(true)
                        .help("tail logs and continue printing new logs until the end of the request, up to 30s")
                )
                .arg(
                Arg::with_name("lines")
                    .long("lines")
                    .default_value("-1")
                    .takes_value(true)
                    .global(true)
                    .help("number of recent log lines file to display. Defaults to -1, unlimited if <=0")
                ).arg(
                    Arg::with_name("since")
                        .long("since")
                        .default_value("0 sec")
                        .takes_value(true)
                        .global(true)
                        .help("return logs newer than a relative duration like 5s in Duration using units defined \
                        in 'https://www.freedesktop.org/software/systemd/man/systemd.time.html#Parsing%20Time%20Spans' \
             --timeout VALUE
             Example: --since 5sec   or --since '5 sec'  'will set timeout to 5 seconds'")
                )
                .args_from_usage(
                    "<NAME> 'function name'
             -k ,--token [token] 'Pass a JWT token to use instead of basic auth'
             -n ,--namespace [namespace]        'Namespace of the function'
             --since-time [since-time]          'include logs since the given timestamp (RFC3339)'
             -o ,--output [output]              'output logs as (plain|keyvalue|json), JSON includes all available keys'
             --time-format  [time-format]         'string format for the timestamp, any value go time format string is allowed, empty will not print the timestamp'
            --name                              'print the function name'
            --instance                          'print the function instance name/id'
            --tls-no-verify 'Disable TLS validation'
            ")
        );
        app
    }
}

impl Logs {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(l_args) = args.subcommand_matches("logs") {
            let gateway = args.value_of("gateway").ok_or(State::Custom(format!(
                "you must set gateway using \
             --gateway, -g http://host"
            )))?;
            let tls_insecure = l_args.is_present("tls-no-verify");
            let token = l_args.value_of("token").unwrap_or("");
            let output_format = l_args.value_of("output").unwrap_or("plain");
            let time_format = l_args.value_of("time-format").unwrap_or("");
            let include_name = l_args.is_present("name");
            let include_instance = l_args.is_present("instance");

            let openfaas_url = std::env::var(OPENFAAS_URL_ENVIRONMENT).unwrap_or(String::new());
            let gateway = get_gateway_url(gateway, DEFAULT_GATEWAY, "", openfaas_url.as_str());
            let msg = check_tls_insecure(gateway.as_str(), tls_insecure);
            if !msg.is_empty() {
                println!("{}", msg);
            }

            let log_request = log_request_from_flags(l_args)?;
            let cli_auth = ClientAuthE::new(token, gateway.as_str())?;
            let client = cli_auth.get_client(gateway.as_str())?;
            let log_events = client.get_logs(log_request).await?;
            //println!("log_events {:?}",log_events);

            let formatter = get_log_formatter(output_format);

            for log_msg in log_events {
                println!(
                    "{}",
                    formatter(&log_msg, time_format, include_name, include_instance)?
                );
            }
            Err(State::Matched)
        } else {
            //todo investigate the output
            Ok(())
        }
    }
}

fn log_request_from_flags<'s>(args: &'s ArgMatches<'s>) -> utility::Result<Request<'s>> {
    let namespace = args.value_of("namespace").unwrap_or("");
    let func_name = args.value_of("NAME").ok_or(utility::Error::Custom(
        "function name is required".to_string(),
    ))?;
    let tail = args.value_of("lines").unwrap_or("-1");
    let tail: isize = tail
        .parse()
        .map_err(|_e| utility::Error::Custom(format!("can't parse {} as integer value", tail)))?;
    let since = args.value_of("since").unwrap_or("0sec");
    let since = parse_duration::parse(since).map_err(|e| utility::Error::Custom(e.to_string()))?;
    let since_time = args.value_of("since-time").unwrap_or("");
    let since_time = if !since_time.is_empty() {
        let since_time = DateTime::parse_from_rfc3339(since_time).map_err(|_e| {
            utility::Error::Custom(format!("can't parse {} as rfc3339 time", since_time))
        })?;
        Duration::from_secs(since_time.timestamp() as u64)
    } else {
        Duration::from_secs(0)
    };

    let since = since_value(since_time, since);
    //todo check this always true
    let mut follow = args.is_present("tail");
    if !follow {
        follow = true;
    }
    Ok(Request {
        name: func_name,
        namespace,
        tail,
        since: Some(since),
        follow,
        ..Default::default()
    })
}

fn since_value(t: Duration, d: Duration) -> i64 {
    if t.as_nanos() != 0 {
        t.as_secs() as i64
    } else if d.as_secs() != 0 {
        let ts = chrono::offset::Local::now();
        let d = d.as_nanos();
        let ts = ts - chrono::Duration::nanoseconds(d as i64);
        ts.timestamp()
    } else {
        0_i64
    }
}
