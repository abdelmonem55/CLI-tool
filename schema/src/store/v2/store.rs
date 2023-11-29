use serde::{Deserialize, Serialize};
use std::collections::HashMap;

///StoreFunction represents a multi-arch function in the store
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct StoreFunction {
    #[serde(default)]
    pub icon: String, //`json:"icon"`
    #[serde(default)]
    pub title: String, //`json:"title"`
    #[serde(default)]
    pub description: String, //`json:"description"`
    #[serde(default)]
    pub name: String, //`json:"name"`
    #[serde(default)]
    pub fprocess: String, //`json:"fprocess"`
    #[serde(default)]
    pub network: String, //`json:"network"`
    #[serde(default)]
    pub repo_url: String, //`json:"repo_url"
    #[serde(rename = "readOnlyRootFilesystem")]
    #[serde(default)]
    pub read_only_root_filesystem: bool, //  `json:"readOnlyRootFilesystem"`
    #[serde(default)]
    pub environment: HashMap<String, String>, //`json:"environment"`
    #[serde(default)]
    pub labels: HashMap<String, String>, //json:"labels"`
    #[serde(default)]
    pub annotations: HashMap<String, String>, //`json:"annotations"`
    #[serde(default)]
    pub images: HashMap<String, String>, //`json:"images"`
}

///GetImageName get image name of function for a platform
impl StoreFunction {
    pub fn get_image_name(&self, platform: &str) -> Option<&String> {
        let image_name = self.images.get(platform);
        image_name
    }
}

/// Store represents an item of store for version 2
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Store {
    pub version: String,               //`json:"version"`
    pub functions: Vec<StoreFunction>, //`json:"functions"`
}
