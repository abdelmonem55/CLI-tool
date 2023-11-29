use crate::client::{add_query_params, Client};
use crate::utils::{NAMESPACE_KEY, SECRET_ENDPOINT};
use reqwest::{Method, StatusCode};
use std::collections::HashMap;
use utility::faas::types::model::Secret;
use utility::{Error, Result};

impl<'s> Client<'s> {
    /// GetSecretList get secrets list
    pub async fn get_secret_list(&self, namespace: &str) -> Result<Vec<Secret>> {
        let mut secret_path = SECRET_ENDPOINT.to_string();
        if !namespace.is_empty() {
            let mut map = HashMap::new();
            map.insert(NAMESPACE_KEY, namespace);
            secret_path = add_query_params(secret_path.as_str(), &map)?;
        }

        let req = self
            .new_request(Method::GET, secret_path.as_str(), "".into())
            .map_err(|e| {
                Error::Custom(format!(
                    "can't open OpenFaaS on URL {}\nand debug reason {}",
                    self.gateway.as_str(),
                    e
                ))
            })?
            .build()
            .map_err(|e| {
                Error::Custom(format!(
                    "can't open OpenFaaS on URL {}\nand debug reason {}",
                    self.gateway.as_str(),
                    e
                ))
            })?;

        let res = self.do_request(req).await.map_err(|e| {
            Error::Custom(format!(
                "can't open OpenFaaS on URL {}\nand debug reason {}",
                self.gateway.as_str(),
                e
            ))
        })?;

        match res.status(){
            StatusCode::OK | StatusCode::ACCEPTED=>{
                let body = res.text().await
                    .map_err(|e| Error::Custom( format!("can't read secrets from OpenFaaS on URL {}\nand debug reason {}", self.gateway.as_str(), e)))?;
                let secrets:Vec<Secret> = serde_json::from_str(body.as_str())
                    .map_err(|e| Error::Custom(format!("can't read secrets from OpenFaaS on URL {}\nand debug reason {}", self.gateway.as_str(), e)))?;
                Ok(secrets)

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

    // UpdateSecret update a secret via the OpenFaaS API by name
    pub async fn update_secret(&self, secret: &Secret) -> Result<(u16, String)> {
        let body = serde_json::to_string(secret).map_err(|e| Error::Custom(format!("{}", e)))?;

        let req = self
            .new_request(Method::PUT, SECRET_ENDPOINT, body)
            .map_err(|e| {
                Error::Custom(format!(
                    "can't open OpenFaaS on URL {}\nand debug reason {}",
                    self.gateway.as_str(),
                    e
                ))
            })?
            .build()
            .map_err(|e| {
                Error::Custom(format!(
                    "can't open OpenFaaS on URL {}\nand debug reason {}",
                    self.gateway.as_str(),
                    e
                ))
            })?;

        let res = self.do_request(req).await.map_err(|e| {
            Error::Custom(format!(
                "can't open OpenFaaS on URL {}\nand debug reason {}",
                self.gateway.as_str(),
                e
            ))
        })?;

        let res=   match res.status(){
            StatusCode::OK | StatusCode::ACCEPTED =>{
                Ok((res.status().as_u16() ,"Updated".into() ))
            },
         StatusCode::NOT_FOUND=>{
             Ok((res.status().as_u16(),format!("unable to find secret: {}", secret.name) ))
         },
            StatusCode::UNAUTHORIZED=>{
                Ok((res.status().as_u16(),format!("unauthorized access, run \"faas-cli login\" to setup authentication for this server") ))
            },
            status=>{
                let body =res.text().await
                .map_err(|e| Error::Custom(format!("can't open OpenFaaS on URL {}\nand debug reason {}", self.gateway.as_str(), e)))?;
                Ok((status.as_u16(),format!("server returned unexpected status code: {} - {}", status,body)))
            }
        };
        res
    }

    ///create secret
    pub async fn remove_secret(&self, secret: &Secret) -> Result<()> {
        let body = serde_json::to_string(secret).map_err(|e| Error::Custom(format!("{}", e)))?;

        let req = self
            .new_request(Method::DELETE, SECRET_ENDPOINT, body)
            .map_err(|e| {
                Error::Custom(format!(
                    "can't open OpenFaaS on URL {}\nand debug reason {}",
                    self.gateway.as_str(),
                    e
                ))
            })?
            .build()
            .map_err(|e| {
                Error::Custom(format!(
                    "can't open OpenFaaS on URL {}\nand debug reason {}",
                    self.gateway.as_str(),
                    e
                ))
            })?;

        let res = self.do_request(req).await.map_err(|e| {
            Error::Custom(format!(
                "can't open OpenFaaS on URL {}\nand debug reason {}",
                self.gateway.as_str(),
                e
            ))
        })?;

        let res=   match res.status(){
            StatusCode::OK | StatusCode::ACCEPTED =>{
                Ok(())
            },
            StatusCode::NOT_FOUND=>{
                Err(Error::Custom(format!("unable to find secret: {}", secret.name) ))
            },
            StatusCode::UNAUTHORIZED=>{
                Err(Error::Custom(format!("unauthorized access, run \"faas-cli login\" to setup authentication for this server") ))
            },
            status=>{
                let body =res.text().await
                    .map_err(|e| Error::Custom(format!("can't open OpenFaaS on URL {}\nand debug reason {}", self.gateway.as_str(), e)))?;
                Err(Error::Custom(format!("server returned unexpected status code: {} - {}", status, body)))
            }
        };
        res
    }

    pub async fn create_secret(&self, secret: &Secret) -> Result<(u16, String)> {
        let body = serde_json::to_string(secret).map_err(|e| Error::Custom(format!("{}", e)))?;

        let req = self
            .new_request(Method::POST, SECRET_ENDPOINT, body)
            .map_err(|e| {
                Error::Custom(format!(
                    "can't open OpenFaaS on URL {}\nand debug reason {}",
                    self.gateway.as_str(),
                    e
                ))
            })?
            .build()
            .map_err(|e| {
                Error::Custom(format!(
                    "can't open OpenFaaS on URL {}\nand debug reason {}",
                    self.gateway.as_str(),
                    e
                ))
            })?;

        let res = self.do_request(req).await.map_err(|e| {
            Error::Custom(format!(
                "can't open OpenFaaS on URL {}\nand debug reason {}",
                self.gateway.as_str(),
                e
            ))
        })?;

        let res=   match res.status(){
            StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED =>{
                Ok((res.status().as_u16() ,"Created".into() ))
            },
            StatusCode::CONFLICT =>{
                Ok((res.status().as_u16(),format!("secret with the name {} already exists\n", secret.name) ))
            },
            StatusCode::UNAUTHORIZED=>{
                Ok((res.status().as_u16(),format!("unauthorized access, run \"faas-cli login\" to setup authentication for this server") ))
            },
            status=>{
                let body =res.text().await
                    .map_err(|e| Error::Custom(format!("can't open OpenFaaS on URL {}\nand debug reason {}", self.gateway.as_str(), e)))?;
                Ok((status.as_u16(),format!("Server returned unexpected status code: {} - {}", status,body)))
            }
        };
        res
    }
}
