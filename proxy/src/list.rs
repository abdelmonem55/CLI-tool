use crate::client::{add_query_params, Client};
use crate::utils::{NAMESPACE_KEY, SYSTEM_PATH};
use reqwest::{Method, StatusCode};
use std::collections::HashMap;
use utility::faas::types::model::FunctionStatus;
use utility::{Error, Result};

impl<'s> Client<'s> {
    /// ListFunctions list deployed functions
    pub async fn list_functions(&mut self, namespace: &str) -> Result<Vec<FunctionStatus>> {
        self.add_check_redirect(|a| {
            a.error(Error::Custom(format!("net/http: use last response")))
        })?;

        let mut list_endpoint = SYSTEM_PATH.to_string();
        if !namespace.is_empty() {
            let mut map = HashMap::new();
            map.insert(NAMESPACE_KEY, namespace);
            list_endpoint = add_query_params(list_endpoint.as_str(), &map)?;
        }

        let req = self
            .new_request(Method::GET, list_endpoint.as_str(), "".into())
            .map_err(|_e| {
                Error::Custom(format!(
                    "can't read list from OpenFaaS on URL {}",
                    self.gateway.as_str(),
                ))
            })?
            .build()
            .map_err(|_e| {
                Error::Custom(format!(
                    "can't read list from OpenFaaS on URL {}",
                    self.gateway.as_str(),
                ))
            })?;

        let res = self.do_request(req).await.map_err(|_e| {
            Error::Custom(format!(
                "can't read list from OpenFaaS on URL {}",
                self.gateway.as_str(),
            ))
        })?;
        match res.status(){
            StatusCode::OK=>{
                let status:Vec<FunctionStatus> = serde_json::from_str(res.text().await?.as_str())
                    .map_err(|_| Error::Custom( format!("can't read list from OpenFaaS on URL {}", self.gateway.as_str())))?;

                Ok(status)


            },
            StatusCode::UNAUTHORIZED=>{
                Err( Error::Custom(format!("unauthorized access, run \"faas-cli login\" to setup authentication for this server") ))
            },
            status=>{
                let err= res.text().await
                    .map(|body| Error::Custom(format!("Server returned unexpected status code {} and body {}", status, body)))?;
                Err(err)
            }
        }
    }
}
