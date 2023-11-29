use crate::client::Client;
use crate::utils::INFO_ENDPOINT;
use reqwest::{Method, StatusCode};
use utility::faas::types::info_request::GatewayInfo;
use utility::{Error, Result};

impl<'s> Client<'s> {
    /// GetSecretList get secrets list
    pub async fn get_system_info(&self) -> Result<GatewayInfo> {
        let req = self
            .new_request(Method::GET, INFO_ENDPOINT, "".into())
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
            StatusCode::OK | StatusCode::ACCEPTED => {
                let body = res.text().await
                    .map_err(|e| Error::Custom(format!("can't read secrets from OpenFaaS on URL {}\nand debug reason {:?}", self.gateway.as_str(), e)))?;
                let info: GatewayInfo = serde_json::from_str(body.as_str())
                    .map_err(|e| Error::Custom(format!("can't read secrets from OpenFaaS on URL {}\nand debug reason {:?}", self.gateway.as_str(), e)))?;
                Ok(info)
            },
            StatusCode::UNAUTHORIZED => {
                Err(Error::Custom(format!("unauthorized access, run \"faas-cli login\" to setup authentication for this server")))
            },
            status => {
                let err = res.text().await
                    .map(|body| Error::Custom(format!("Server returned unexpected status code {} and body {}", status, body)))?;
                Err(err)
            }
        }
    }
}
