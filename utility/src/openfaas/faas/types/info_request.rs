use crate::openfaas::faas_provider::types::{ProviderInfo, VersionInfo};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

lazy_static! {
/// Platform architecture the gateway is running on
    static ref ARCH:Arc<Mutex<String>>  = Arc::new(Mutex::new(String::new()));
    }

// GatewayInfo provides information about the gateway and it's connected components
#[derive(Serialize, Deserialize, Debug, PartialEq, Default)]
pub struct GatewayInfo {
    #[serde(default)]
    pub provider: ProviderInfo, //`json:"provider"`
    #[serde(default)]
    pub version: VersionInfo, //`json:"version"`
    #[serde(default)]
    pub arch: String, //`json:"arch"`
}
