use crate::client::{add_query_params, Client};
use crate::utils::{NAMESPACE_KEY, SCALE_PATH};
use reqwest::{Method, StatusCode};
use std::collections::HashMap;
use utility::faas_provider::types::ScaleServiceRequest;
use utility::{Error, Result};

impl<'s> Client<'s> {
    ///ScaleFunction scale a function
    pub async fn scale_function(
        &self,
        function_name: &str,
        namespace: &str,
        replicas: u64,
    ) -> Result<()> {
        let scale_req = ScaleServiceRequest {
            service_name: function_name,
            replicas,
        };

        let body =
            serde_json::to_string(&scale_req).map_err(|e| Error::Custom(format!("{:?}", e)))?;
        let mut function_path = SCALE_PATH.trim_end_matches('/').to_string()
            + "/"
            + function_name.trim_start_matches("/");

        if !namespace.is_empty() {
            let mut map = HashMap::new();
            map.insert(NAMESPACE_KEY, namespace);
            function_path = add_query_params(function_path.as_str(), &map)?;
        }

        let req = self
            .new_request(Method::POST, function_path.as_str(), body)
            .map_err(|e| {
                Error::Custom(format!(
                    "can't open OpenFaaS on URL {}\nand debug reason {:?}",
                    self.gateway.as_str(),
                    e
                ))
            })?
            .build()
            .map_err(|e| {
                Error::Custom(format!(
                    "can't open OpenFaaS on URL {}\nand debug reason {:?}",
                    self.gateway.as_str(),
                    e
                ))
            })?;

        let res = self.do_request(req).await.map_err(|e| {
            Error::Custom(format!(
                "can't open OpenFaaS on URL {}\nand debug reason {:?}",
                self.gateway.as_str(),
                e
            ))
        })?;

        match res.status() {
            StatusCode::OK | StatusCode::ACCEPTED | StatusCode::CREATED => {
                Ok(())
            },
            StatusCode::UNAUTHORIZED => {
                Err(Error::Custom(format!("unauthorized access, run \"faas-cli login\" to setup authentication for this server")))
            },
            StatusCode::NOT_FOUND => {
                Err(Error::Custom(format!("No such function: {}", function_name)))
            },
            status => {
                let err = res.text().await
                    .map(|body| Error::Custom(format!("Server returned unexpected status code {} and body {}", status, body)))?;
                Err(err)
            }
        }
    }
}
