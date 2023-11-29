#![allow(dead_code)]
use crate::store_deploy::StoreDeploy;
use crate::store_describe::StoreDescribe;
use crate::store_list::StoreList;
use crate::template_store_list::MAIN_PLATFORM;
use crate::{CommandAppend, State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};
use proxy::proxy::make_http_client;
use reqwest::StatusCode;
use schema::store::v2::store::{Store as V2Store, StoreFunction};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use utility::{Error, Result};

pub(crate) const DEFAULT_STORE: &str =
    "https://raw.githubusercontent.com/openfaas/store/master/functions.json";
pub(crate) const MAX_DESCRIPTION_LEN: usize = 40;

pub(crate) const PLATFORM: &str = "";

lazy_static::lazy_static! {
    //pub static ref PLATFORM:Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref SHORT_PLATFORM:Arc<HashMap<&'static str,&'static str>> = {
     let mut map = HashMap::new();
     map.insert("linux/arm/v6", "armhf");
     map.insert("linux/amd64",  "x86_64");
     map.insert("linux/arm64", "arm64");
        Arc::new(map)
        };
}

pub(crate) struct Store;

impl CommandAppend for Store {
    #[inline(always)]
    fn append_subcommand(app: App<'static, 'static>) -> App<'static, 'static> {
        let app = app.subcommand(
            SubCommand::with_name("store")
                .about(r#"Allows browsing and deploying OpenFaaS functions from a store`"#)
                .arg(
                    Arg::with_name("url")
                        .long("url")
                        .short("u")
                        .takes_value(true)
                        .global(true)
                        .default_value(DEFAULT_STORE)
                        .help("Alternative Store URL starting with http(s)://"),
                )
                .arg(
                    Arg::with_name("platform")
                        .long("platform")
                        .short("p")
                        .takes_value(true)
                        .global(true)
                        .help("Target platform for store"),
                )
                //add subcommands like store
                .subcommand(StoreList::append_subcommand()) //.subcommand(TemplatePull::append_subcommand(app_copy.clone())),
                .subcommand(StoreDeploy::append_subcommand())
                .subcommand(StoreDescribe::append_subcommand()),
        );
        app
    }
}

impl Store {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(s_args) = args.subcommand_matches("store") {
            StoreList::dispatch_command(s_args).await?;
            StoreDeploy::dispatch_command(s_args).await?;
            StoreDescribe::dispatch_command(s_args).await?;
            //   TemplatePull::dispatch_command(t_args).await?;

            Err(State::Custom(
                r#"store command must followed by sub command for example:
             faas-cli store describe
             faas-cli store list
             for help type faas-cli store --help
             "#
                .to_string(),
            ))
        } else {
            Ok(())
        }
    }
}

pub(crate) async fn store_list(store: &str) -> Result<Vec<StoreFunction>> {
    let store = store.trim_end_matches('/');

    let timeout = Duration::from_secs(60);
    //timeout := 60 * time.Second
    let tls_insecure = false;
    let client = make_http_client(Some(timeout), tls_insecure)?;

    let req = client.get(store).build()?;
    let res = client.execute(req).await.map_err(|_e| {
        Error::Custom(format!(
            "cannot connect to OpenFaaS store at URL: {}",
            store
        ))
    })?;
    match res.status() {
        StatusCode::OK => {
            let body = res.text().await.map_err(|_e| {
                Error::Custom(format!(
                    "cannot connect to OpenFaaS store at URL: {}",
                    store
                ))
            })?;
            let store_data: V2Store = serde_json::from_str(body.as_str()).map_err(|e| {
                Error::Custom(format!(
                    "cannot parse result from OpenFaaS store at URL: {}\n{}",
                    store, e
                ))
            })?;
            Ok(store_data.functions)
        }
        status => {
            let body = res.text().await.map_err(|_e| {
                Error::Custom(format!(
                    "cannot connect to OpenFaaS store at URL: {}",
                    store
                ))
            })?;
            Err(Error::Custom(format!(
                "server returned unexpected status code: {} - {}",
                status, body
            )))
        }
    }
}

pub(crate) fn filter_store_list(
    functions: Vec<StoreFunction>,
    platform: &str,
) -> Vec<StoreFunction> {
    let mut filtered_list: Vec<StoreFunction> = Vec::new();

    for function in functions {
        let (_, ok) = get_value_ignore_case(&function.images, platform);

        if ok {
            filtered_list.push(function);
        }
    }

    filtered_list
}

//get a key value from map by ignoring case for key
fn get_value_ignore_case(kv: &HashMap<String, String>, key: &str) -> (String, bool) {
    for (k, v) in kv {
        if k.eq_ignore_ascii_case(key) {
            return (v.to_string(), true);
        }
    }

    (String::new(), false)
}

pub(crate) fn get_platform() -> &'static str {
    if !PLATFORM.is_empty() {
        PLATFORM
    } else {
        MAIN_PLATFORM
    }
}

pub(crate) fn get_target_platform(input_platform: &str) -> String {
    if input_platform.is_empty() {
        let current_platform = get_platform();

        SHORT_PLATFORM
            .get(current_platform)
            .map(|t| t.to_string())
            .unwrap_or(current_platform.to_string())
    } else {
        input_platform.to_string()
    }
}

pub(crate) fn get_store_platforms(functions: &Vec<StoreFunction>) -> Vec<String> {
    let mut distinct_platform_map: HashMap<String, bool> = HashMap::new();
    let mut result: Vec<String> = Vec::new();

    for function in functions {
        for (key, _) in &function.images {
            if distinct_platform_map.get(key.as_str()).is_none() {
                distinct_platform_map.insert(key.clone(), true);
                result.push(key.clone());
            }
        }
    }

    return result;
}

pub(crate) fn store_find_function<'s>(
    function_name: &str,
    store_items: &'s Vec<StoreFunction>,
) -> Option<&'s StoreFunction> {
    for item in store_items {
        if item.name == function_name || item.title == function_name {
            return Some(item);
        }
    }
    None
}
