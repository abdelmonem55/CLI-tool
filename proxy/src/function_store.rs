use schema::store::v2::store::StoreFunction;
use serde::Deserialize;
use utility::{Error, Result};

#[derive(Deserialize, Default, Debug)]
pub struct StoreResult {
    _version: String,               //`json:"version"`
    functions: Vec<StoreFunction>, //`json:"functions"`
}

/// returns functions from a store URL
pub async fn function_store_list(store: &str) -> Result<Vec<StoreFunction>> {
    let store = store.trim_end_matches('/');
    let resp = reqwest::get(store).await?;

    let res = match resp.status() {
        reqwest::StatusCode::OK => {
            let body = resp.text().await?;
            let store_res: StoreResult = serde_yaml::from_str(body.as_str())
                .map_err(|e| Error::Custom(format!("{:?}", e)))?;
            Ok(store_res.functions)
        }
        _ => Err(Error::Custom(format!(
            "expected status is ok but found {}",
            resp.status()
        ))),
    };
    res
}
