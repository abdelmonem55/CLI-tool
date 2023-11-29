use crate::client::{add_query_params, Client};
use crate::utils::{NAMESPACE_KEY, SYSTEM_PATH};
use http::StatusCode;
use reqwest::Method;
use utility::faas_provider::types::DeleteFunctionRequest;
use utility::{Error, Result};

impl<'s> Client<'s> {
    //DeleteFunction delete a function from the OpenFaaS server
    pub async fn delete_function(&self, function_name: &str, namespace: &str) -> Result<()> {
        let del_req = DeleteFunctionRequest {
            function_name: function_name,
        };

        let req_data =
            serde_json::to_string(&del_req).map_err(|e| Error::Custom(format!("{:?}", e)))?;

        let mut delete_endpoint = SYSTEM_PATH.to_string();
        if !namespace.is_empty() {
            delete_endpoint = add_query_params(
                delete_endpoint.as_str(),
                &[(NAMESPACE_KEY, namespace)].iter().cloned().collect(),
            )?;
        }

        let req = self
            .new_request(Method::DELETE, delete_endpoint.as_str(), req_data.clone())
            .map_err(|e| {
                Error::Custom(format!(
                    "can't delete function from OpenFaaS on URL {}\nand debug reason {:?}",
                    self.gateway.as_str(),
                    e
                ))
            })?
            .build()
            .map_err(|e| {
                Error::Custom(format!(
                    "can't delete function from OpenFaaS on URL {}\nand debug reason {:?}",
                    self.gateway.as_str(),
                    e
                ))
            })?;
        let resp = self.do_request(req).await?;

        let res=  match resp.status(){
            StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED =>{
                println!("Removing old function..");
                Ok(())
            },
          StatusCode::NOT_FOUND=>{
              Err(Error::Custom(format!("(No existing function to remove")))
          },

          StatusCode::UNAUTHORIZED =>{
              Err(Error::Custom(format!("unauthorized access, run \"faas-cli login\" to setup authentication for this server")))
          },
          status=>{
             let err= resp.text().await
                  .map(|body| Error::Custom(format!("Server returned unexpected status code {} and body {}", status, body)))?;
              Err(err)
          }
        };

        res
    }
}
