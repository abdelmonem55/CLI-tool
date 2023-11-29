use crate::Error;
use regex::Regex;
use std::collections::HashMap;

///wrapper function above envsubst to handle default values in form of ${VAR:-default}
pub fn substitute(data: &str, vars: &HashMap<String, String>) -> crate::Result<String> {
    let start = regex::escape("${");
    let end = regex::escape("}");

    let regex = Regex::new(format!(r#"{}[a-zA-Z_][a-zA-Z0-9_]*:?-(.*?){}"#, start, end).as_str())
        .map_err(|e| Error::Custom(format!("{:?}", e)))?;
    let mut format = data.to_string();
    let capts = regex.captures_iter(data);
    //let vars:HashMap<String,String> = vars().collect();
    for cap in capts {
        let default = &cap[1];
        //remove :-default
        let key = cap[0]
            .replace(default, "")
            .replace("${", "")
            .replace("}", "")
            .replace(":-", "");
        if let Some(val) = vars.get(key.as_str()) {
            if val.is_empty() {
                format = format.replace(&cap[0], default);
            } else {
                format = format.replace(&cap[0], val);
            }
        } else {
            format = format.replace(&cap[0], default);
        }

        // println!("{:?},  {}",cap,default);
    }
    // envsubst::substitute(format,&vars)
    //     .map_err(|e| Error::IoCustom(format!("{:?}",e)))

    let regex = Regex::new(format!(r#"{}[a-zA-Z_][a-zA-Z0-9_]*{}?"#, start, end).as_str())
        .map_err(|e| Error::Custom(format!("{:?}", e)))?;
    let data = format.clone();
    let capts = regex.captures_iter(data.as_str());
    //let vars:HashMap<String,String> = vars().collect();
    //catch envs with default values
    for cap in capts {
        // let default = &cap[1];
        //remove :-default
        let key = cap[0].replace("${", "").replace("}", "");
        if let Some(val) = vars.get(key.as_str()) {
            format = format.replace(&cap[0], val);
        } else {
            format = format.replace(&cap[0], "");
        }

        // println!("{:?},  {}",cap,default);
    }
    Ok(format)
}
