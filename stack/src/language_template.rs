use crate::schema::LanguageTemplate;
use std::fs::read_to_string;
use utility::{Error, Result};

pub async fn parse_yaml_for_language_template(file: &str) -> Result<LanguageTemplate> {
    let url_res = url::Url::parse(file);
    if let Ok(url) = url_res {
        if !url.scheme().is_empty() {
            //println!("url parsed {}",url.to_string());
            let file_data = crate::stack::fetch_yaml(url).await?;
            return parse_yaml_data_for_language_template(file_data.as_str());
        }
    }
    let file_data = read_to_string(file)?;
    return parse_yaml_data_for_language_template(file_data.as_str());
}

/// parse_yaml_data_for_language_template parses YAML data into language template
pub fn parse_yaml_data_for_language_template(file: &str) -> Result<LanguageTemplate> {
    serde_yaml::from_str(file).map_err(|e| {
        println!("Error with YAML file");
        Error::Custom(format!("{:?}", e))
    })
}

pub async fn is_valid_template(lang: &str) -> bool {
    let lang = lang.to_ascii_lowercase();
    let mut found = false;
    if std::fs::metadata(format!("./template/{}", lang)).is_ok() {
        let template_yaml_path = format!("./template/{}/template.yml", lang);
        if parse_yaml_for_language_template(template_yaml_path.as_str())
            .await
            .is_ok()
        {
            found = true;
        }
    }
    found
}

///LoadLanguageTemplate loads language template details from template.yml file.
pub async fn load_language_template(lang: &str) -> Result<LanguageTemplate> {
    let lang = lang.to_ascii_lowercase();
    std::fs::metadata(format!("./template/{}", lang))?;
    let template_yaml_path = format!("./template/{}/template.yml", lang);
    parse_yaml_for_language_template(template_yaml_path.as_str()).await
    //assert that lang is exits
}
