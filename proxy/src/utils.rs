use std::path::Path;
use utility::{Error, Result};

pub const SYSTEM_PATH: &str = "/system/functions";
pub const FUNCTION_PATH: &str = "/system/function";
pub const NAMESPACES_PATH: &str = "/system/namespaces";
pub const NAMESPACE_KEY: &str = "namespace";
pub const SCALE_PATH: &str = "/system/scale-function";
pub const LOGS_PATH: &str = "/system/logs";
pub const SECRET_ENDPOINT: &str = "/system/secrets";
pub const INFO_ENDPOINT: &str = "/system/info";

pub fn create_system_endpoint(gateway: &str, namespace: &str) -> Result<String> {
    let mut url = url::Url::parse(gateway).map_err(|e| Error::Custom(format!("{:?}", e)))?;
    url.set_path(SYSTEM_PATH);
    if !namespace.is_empty() {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("namespace", namespace);
    }
    Ok(url.to_string())
}

pub fn create_function_endpoint(
    gateway: &str,
    function_name: &str,
    namespace: &str,
) -> Result<String> {
    let mut url = url::Url::parse(gateway).map_err(|e| Error::Custom(format!("{:?}", e)))?;
    let path = Path::new(FUNCTION_PATH);
    let path = path.join(function_name);
    let path = path
        .to_str()
        .ok_or(Error::Custom(format!("non utf8 url")))?;
    url.set_path(path);
    if !namespace.is_empty() {
        let mut pairs = url.query_pairs_mut();
        pairs.append_pair("namespace", namespace);
    }
    Ok(url.to_string())
}

// func createNamespacesEndpoint(gateway string) (string, error) {
// gatewayURL, err := url.Parse(gateway)
// if err != nil {
// return "", fmt.Errorf("invalid gateway URL: %s", err.Error())
// }
// gatewayURL.Path = path.Join(gatewayURL.Path, namespacesPath)
// return gatewayURL.String(), nil
// }
