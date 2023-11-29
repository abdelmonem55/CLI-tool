use serde::{Deserialize, Serialize};
use std::collections::HashMap;
///Metadata metadata of the object
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Metadata {
    #[serde(default)]
    #[serde(skip_serializing_if = "utility::is_default")]
    pub name: String, //`yaml:"name,omitempty"`
    #[serde(default)]
    #[serde(skip_serializing_if = "utility::is_default")]
    pub namespace: String, //`yaml:"namespace,omitempty"`
    #[serde(default)]
    #[serde(skip_serializing_if = "utility::is_default")]
    pub annotations: HashMap<String, String>, //`yaml:"annotations,omitempty"`
}
