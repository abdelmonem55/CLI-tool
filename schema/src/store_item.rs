use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// StoreItem represents an item of store

#[derive(Serialize, Deserialize, Debug)]
pub struct StoreItem {
    pub icon: String,                         //`json:"icon"`
    pub title: String,                        //`json:"title"`
    pub description: String,                  //`json:"description"`
    pub image: String,                        //`json:"image"`
    pub name: String,                         //`json:"name"`
    pub fprocess: String,                     //`json:"fprocess"`
    pub network: String,                      //`json:"network"`
    pub repo_url: String,                     //`json:"repo_url"`
    pub environment: HashMap<String, String>, //`json:"environment"`
    pub labels: HashMap<String, String>,      //`json:"labels"`
    pub annotations: HashMap<String, String>, //`json:"annotations"`
    #[serde(rename = "readOnlyRootFilesystem")]
    pub read_only_root_filesystem: bool, //  `json:"readOnlyRootFilesystem"`
}
