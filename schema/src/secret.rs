use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct KubernetesSecret {
    #[serde(default)]
    pub kind: String, // `json:"kind"`

    #[serde(rename = "apiVersion")]
    #[serde(default)]
    pub api_version: String, //`json:"apiVersion"`

    #[serde(default)]
    pub metadata: KubernetesSecretMetadata, //`json:"metadata"`

    #[serde(default)]
    pub data: HashMap<String, String>, //json:"data"`
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct KubernetesSecretMetadata {
    #[serde(default)]
    pub name: String, // `json:"name"`
    #[serde(default)]
    pub namespace: String, // `json:"namespace"`
}
