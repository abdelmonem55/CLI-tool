use serde::{Deserialize, Serialize};
///scales the service to the requested replcia count.
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct ScaleServiceRequest<'s> {
    #[serde(rename = "serviceName")]
    pub service_name: &'s str, // `json:"serviceName"`
    pub replicas: u64, //`json:"replicas"`
}

/// DeleteFunctionRequest delete a deployed function
#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub struct DeleteFunctionRequest<'s> {
    #[serde(rename = "functionName")]
    pub function_name: &'s str, // `json:"functionName"`
}

/// ProviderInfo provides information about the configured provider
#[derive(Serialize, Deserialize, PartialEq, Clone, Default, Debug)]
pub struct ProviderInfo {
    #[serde(rename = "provider")]
    pub name: String, //       `json:"provider"`
    pub version: VersionInfo,  //`json:"version"`
    pub orchestration: String, //       `json:"orchestration"`
}

/// VersionInfo provides the commit message, sha and release version number
#[derive(Serialize, Deserialize, PartialEq, Clone, Default, Debug)]
pub struct VersionInfo {
    #[serde(default)]
    pub commit_message: String, // `json:"commit_message,omitempty"`
    pub sha: String,     // `json:"sha"`
    pub release: String, // `json:"release"`
}
