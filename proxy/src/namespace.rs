use crate::client::Client;
use crate::utils::NAMESPACES_PATH;
use reqwest::{Method, StatusCode};
use utility::{Error, Result};

impl<'s> Client<'s> {
    ///lists available function namespaces
    pub async fn list_namesapces(&mut self) -> Result<Vec<String>> {
        self.add_check_redirect(|a| {
            a.error(Error::Custom(format!("net/http: use last response")))
        })?;

        let req = self
            .new_request(Method::GET, NAMESPACES_PATH, "".into())
            .map_err(|_e| {
                Error::Custom(format!(
                    "can't read namespaces from OpenFaaS on URL {}",
                    self.gateway.as_str(),
                ))
            })?
            .build()
            .map_err(|_e| {
                Error::Custom(format!(
                    "can't read namespaces from OpenFaaS on URL {}",
                    self.gateway.as_str(),
                ))
            })?;

        let res = self.do_request(req).await.map_err(|_e| {
            Error::Custom(format!(
                "can't read namespaces from OpenFaaS on URL {}",
                self.gateway.as_str(),
            ))
        })?;

        match res.status(){
            StatusCode::OK=>{
                let body = res.text().await
                    .map_err(|_e| {
                        Error::Custom(format!(
                            "can't read namespaces from OpenFaaS on URL {}",
                            self.gateway.as_str(),
                        ))
                    })?;
                let status:Vec<String> = serde_json::from_str(body.as_str())
                    .map_err(|_e| {
                        Error::Custom(format!(
                            "can't read namespaces from OpenFaaS on URL {}",
                            self.gateway.as_str(),
                        ))
                    })?;
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
