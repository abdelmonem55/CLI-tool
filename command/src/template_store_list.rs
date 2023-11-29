use crate::priority::{get_template_store_url, TEMPLATE_STORE_URL_ENVIRONMENT};
use crate::{State, SubCommandAppend};
use clap::{App, Arg, ArgMatches, SubCommand};
use reqwest::StatusCode;
use serde::Deserialize;
use utility::{Error, Result};

///DEFAULT_TEMPLATE_STORE is the URL where the official store can be found
pub(crate) const DEFAULT_TEMPLATE_STORE: &str =
    "https://raw.githubusercontent.com/openfaas/store/master/templates.json";
pub(crate) const MAIN_PLATFORM: &str = "x86_64";

// "template": "go",
// "platform": "x86_64",
// "language": "Go",
// "source": "openfaas",
// "description": "Classic Golang template",
// "repo": "https://github.com/openfaas/templates",
// "official": "true"
/// TemplateInfo is the definition of a template which is part of the store
#[derive(Deserialize)]
pub(crate) struct TemplateInfo {
    #[serde(rename = "template")]
    pub(crate) template_name: String, // `json:"template"`
    pub(crate) platform: String,    // `json:"platform"`
    pub(crate) language: String,    // `json:"language"`
    pub(crate) source: String,      // `json:"source"`
    pub(crate) description: String, // `json:"description"`
    #[serde(rename = "repo")]
    pub(crate) repository: String, // `json:"repo"`
    pub(crate) official: String,    // `json:"official"`
}

pub(crate) struct TemplateStoreList;

impl SubCommandAppend for TemplateStoreList {
    #[inline(always)]
    fn append_subcommand() -> App<'static, 'static> {
        let app =
            SubCommand::with_name("list")
                .alias("ls")
                .about(r#"List templates from official store or from custom URL or set the environmental variable OPENFAAS_TEMPLATE_STORE_URL to be the default store location`,
	Example: `  faas-cli template store list
  faas-cli template store ls
  faas-cli template store ls --url=https://raw.githubusercontent.com/openfaas/store/master/templates.json
  faas-cli template store ls --verbose=true
  faas-cli template store list --platform arm64`"#)
                .arg(
                    Arg::with_name("platform")
                        .long("platform")
                        .short("p")
                        .global(true)
                        .takes_value(true)
                        .default_value(MAIN_PLATFORM)
                        .help("Shows the platform if the output is verbose"),
                )
                .arg(
                    Arg::with_name("url")
                        .long("url")
                        .short("u")
                        .global(true)
                        .takes_value(true)
                        .default_value(DEFAULT_TEMPLATE_STORE)
                        .help("Path to YAML file describing function(s)"),
                );
        app
    }
}

impl TemplateStoreList {
    #[inline(always)]
    pub(crate) async fn dispatch_command(args: &ArgMatches<'_>) -> crate::Result {
        if let Some(l_args) = args.subcommand_matches("list") {
            let template_store_url = l_args.value_of("url").unwrap_or(DEFAULT_TEMPLATE_STORE);
            let platform = l_args.value_of("platform").unwrap_or(MAIN_PLATFORM);
            let verbose = l_args.is_present("verbose");

            let env_template_repo_store =
                std::env::var(TEMPLATE_STORE_URL_ENVIRONMENT).unwrap_or_default();

            let store_url = get_template_store_url(
                template_store_url,
                env_template_repo_store.as_str(),
                DEFAULT_TEMPLATE_STORE,
            );

            //println!("store {}",store_url);
            let template_info = get_template_info(store_url.as_str())
                .await
                .map_err(|e| State::Custom(format!("error while template info: {}", e)))?;

            let formatted_output =
                format_templates_output(template_info, verbose, platform).unwrap_or_default();
            colour::green!("{}", formatted_output);

            Err(State::Matched)
        } else {
            Ok(())
        }
    }
}

pub(crate) async fn get_template_info(repository: &str) -> Result<Vec<TemplateInfo>> {
    let res = reqwest::get(repository)
        .await
        .map_err(|e| Error::Custom(format!("error while requesting template list: {}", e)))?;
    /* req, reqErr := http.NewRequest(http.MethodGet, repository, nil)
    if reqErr != nil {
        return nil, fmt.Errorf("error while trying to create request to take template info: %s", reqErr.Error())
    }

    reqContext, cancel := context.WithTimeout(req.Context(), 5*time.Second)
    defer cancel()
    req = req.WithContext(reqContext)

    client := http.DefaultClient
    res, clientErr := client.Do(req)
    if clientErr != nil {
        return nil, fmt.Errorf("error while requesting template list: %s", clientErr.Error())
    }

    if res.Body == nil {
        return nil, fmt.Errorf("error empty response body from: %s", templateStoreURL)
    }
    defer res.Body.Close()*/
    if res.status() == StatusCode::OK {
        let body = res.text().await.map_err(|e| {
            Error::Custom(format!(
                "error while reading data from templates body: {}",
                e
            ))
        })?;
        if !body.is_empty() {
            // println!("{}",body);
            let res = serde_json::from_str(body.as_str()).map_err(|e| {
                Error::Custom(format!(
                    "error while deserialize into templates struct: {}",
                    e
                ))
            })?;
            Ok(res)
        } else {
            Err(Error::Custom("empty body".to_string()))
        }
    } else {
        Err(Error::Custom(format!(
            "unexpected status code wanted: {} got: {}",
            StatusCode::OK,
            res.status()
        )))
    }
}

fn format_templates_output(
    templates: Vec<TemplateInfo>,
    verbose: bool,
    platform: &str,
) -> Option<String> {
    let templates = if platform != MAIN_PLATFORM {
        filter_template(templates, platform)
    } else {
        filter_template(templates, MAIN_PLATFORM)
    };

    if !templates.is_empty() {
        if verbose {
            Some(format_verbose_output(templates))
        } else {
            Some(format_basic_output(templates))
        }
    } else {
        None
    }
}

fn filter_template(templates: Vec<TemplateInfo>, platform: &str) -> Vec<TemplateInfo> {
    let mut filter_templates = Vec::new();

    for template in templates {
        if template.platform.eq_ignore_ascii_case(platform) {
            filter_templates.push(template);
        }
    }
    filter_templates
}

fn format_basic_output(templates: Vec<TemplateInfo>) -> String {
    let width = 40;
    let mini_width = 25;

    let mut str = format!("{:width$}", "NAME", width = width)
        + format!("{:width$}", "SOURCE", width = mini_width).as_str()
        + "DESCRIPTION\n";
    for template in templates {
        let temp = format!("{:width$}", template.template_name, width = width)
            + format!("{:width$}", template.source, width = mini_width).as_str()
            + format!("{}\n", template.description).as_str();
        str.push_str(temp.as_str());
    }
    str
}

fn format_verbose_output(templates: Vec<TemplateInfo>) -> String {
    let width = 40;
    let mini_width = 15;

    let mut str = format!("{:width$}", "NAME", width = width)
        + format!("{:width$}", "LANGUAGE", width = mini_width).as_str()
        + format!("{:width$}", "PLATFORM", width = mini_width).as_str()
        + format!("{:width$}", "SOURCE", width = 25).as_str()
        + "DESCRIPTION\n";
    for template in templates {
        let temp = format!("{:width$}", template.template_name, width = width)
            + format!("{:width$}", template.language, width = mini_width).as_str()
            + format!("{:width$}", template.platform, width = mini_width).as_str()
            + format!("{:width$}", template.source, width = 25).as_str()
            + format!("{}\n", template.description).as_str();

        str.push_str(temp.as_str());
    }
    str
}
