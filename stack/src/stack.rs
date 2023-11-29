use crate::schema::Services;
use lazy_static::lazy_static;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use utility::{Error, Result};

//const LEGACY_PROVIDER_NAME:&str = "faas";
const PROVIDER_NAME: &str = "openfaas";
//const DEFAULT_SCHEMA_VERSION:&str = "1.0";

lazy_static! {
/// ValidSchemaVersions available schema versions
 static ref VALID_SHEMA_VERSION:Arc<Vec<&'static str>> =Arc::new(vec!["1.0"]);
}
pub struct ValidSchemaDisplay(Arc<Vec<&'static str>>);
impl Debug for ValidSchemaDisplay {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut data = String::new();
        for str in self.0.iter() {
            data += str;
        }
        write!(f, "{}", data)
    }
}

///fetchYAML pulls in file from remote location such as GitHub raw file-view
pub(crate) async fn fetch_yaml(address: url::Url) -> Result<String> {
    Ok(reqwest::get(address.as_str()).await?.text().await?)
}

/// IsValidSchemaVersion validates schema version
pub fn is_valid_schema_version(schema_version: &str) -> bool {
    for version in VALID_SHEMA_VERSION.iter() {
        if schema_version == *version {
            return true;
        }
    }
    false
}

/// ParseYAMLFile parse YAML file into a tests of "services".
pub async fn parse_yaml_file(
    yaml_file: &str,
    regex: &str,
    filter: &str,
    envsubst: bool,
) -> Result<Services> {
    let url = url::Url::parse(yaml_file).map_err(|e| Error::Custom(format!("{:?}", e)));

    let data = if url.is_ok()
        && !url.as_ref().unwrap().scheme().is_empty()
        && url.as_ref().unwrap().host().is_some()
    {
        let url = url.unwrap();
        fetch_yaml(url).await?
    } else {
        std::fs::read_to_string(yaml_file)?
    };
    parse_yaml_data(data.as_str(), regex, filter, envsubst)
}
pub fn substitute_vars(data: &str) -> Result<String> {
    let vars = std::env::vars().collect();
    utility::envsubst::substitute(data, &vars).map_err(|e| Error::Custom(format!("{:?}", e)))
}
/// ParseYAMLData parse YAML data into a tests of "services".
pub fn parse_yaml_data(data: &str, regex: &str, filter: &str, envsubsts: bool) -> Result<Services> {
    let regex_exists = regex.len() > 0;
    let filter_exists = filter.len() > 0;
    let data = if envsubsts {
        // let vars = std::env::vars().collect();
        // let subst_data = envsubst::substitute(data,&vars)
        //     .map_err(|e| Error::IoCustom(format!("{:?}",e)))?;
        let subst_data = substitute_vars(data)?;
        subst_data
    } else {
        data.to_owned()
    };
    let mut services: Services =
        serde_yaml::from_str(data.as_str()).map_err(|e| Error::Custom(format!("{:?}", e)))?;
    // let mut services = services_old.clone();
    for (_, mut f) in &mut services.functions {
        if f.language.as_str() == "Dockerfile" {
            f.language = "dockerfile".into();
            // services.functions.insert(key,f);
        }
    }
    if services.provider.name != PROVIDER_NAME {
        return Err(Error::Custom(format!(
            "['{}'] is the only valid 'provider.name' for the OpenFaaS CLI, but you gave: {}",
            PROVIDER_NAME, services.provider.name
        )));
    }

    if !services.version.is_empty() && !is_valid_schema_version(services.version.as_str()) {
        return Err(Error::Custom(format!(
            "{:?} are the only valid versions for the tests file - found: {}",
            ValidSchemaDisplay(VALID_SHEMA_VERSION.clone()),
            &services.version
        )));
    }

    if regex_exists && filter_exists {
        return Err(Error::Custom(format!(
            "pass in a regex or a filter, not both"
        )));
    }

    let old_services = services;
    let mut services = old_services.clone();
    if regex_exists || filter_exists {
        for (k, mut function) in old_services.functions {
            // let mut is_match =false;
            //let mut function = function.clone();
            function.name = k.to_string();

            let is_match = if regex_exists {
                let regex =
                    regex::Regex::new(regex).map_err(|e| Error::Custom(format!("{:?}", e)))?;
                regex.is_match(function.name.as_str())
            } else {
                let mat = wildmatch::WildMatch::new(filter);
                mat.matches(function.name.as_str())
            };

            if !is_match {
                // delete(services.Functions, function.Name)
                services.functions.remove(function.name.as_str());
            }
        }

        if services.functions.is_empty() {
            return Err(Error::Custom(String::from(
                "no functions matching --filter/--regex were found in the YAML file",
            )));
        }
    }

    Ok(services)
}
