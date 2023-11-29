use crate::client::{add_query_params, Client};
use crate::utils::{FUNCTION_PATH, NAMESPACE_KEY};
use http::StatusCode;
use reqwest::redirect::{Action, Attempt};
use reqwest::Method;
use std::collections::HashMap;
use utility::faas::types::model::FunctionStatus;
use utility::{Error, Result};

///get an OpenFaaS function information
impl<'s> Client<'s> {
    pub async fn get_function_info(
        &self,
        function_name: &str,
        namespace: &str,
    ) -> Result<FunctionStatus> {
        let mut function_path = format!(
            "{}/{}/{}",
            self.gateway.as_str().trim_end_matches('/'),
            FUNCTION_PATH,
            function_name
        );
        if !namespace.is_empty() {
            let mut map = HashMap::new();
            map.insert(NAMESPACE_KEY, namespace);
            function_path = add_query_params(function_path.as_str(), &map)?;
        }

        let req = self
            .new_request(Method::GET, function_path.as_str(), "".into())
            .map_err(|e| {
                Error::Custom(format!(
                    "can't read function info from OpenFaaS on URL {}\nand debug reason {:?}",
                    self.gateway.as_str(),
                    e
                ))
            })?
            .build()
            .map_err(|e| {
                Error::Custom(format!(
                    "can't read function info from OpenFaaS on URL {}\nand debug reason {:?}",
                    self.gateway.as_str(),
                    e
                ))
            })?;

        let res = self.do_request(req).await.map_err(|_| {
            Error::Custom(format!(
                "can't connect to OpenFaaS on URL: {}",
                self.gateway.as_str()
            ))
        })?;
        match res.status(){
            StatusCode::OK=>{
                let status:FunctionStatus = serde_json::from_str(res.text().await?.as_str())
                    .map_err(|e| Error::Custom( format!("can't read function info from OpenFaaS on URL {}\nand debug reason {:?}", self.gateway.as_str(), e)))?;
                Ok(status)


            },
            StatusCode::UNAUTHORIZED=>{
                 Err( Error::Custom(format!("unauthorized access, run \"faas-cli login\" to setup authentication for this server") ))
            },
            StatusCode::NOT_FOUND=>{
                 Err( Error::Custom(format!("No such function: {}", function_name)))
            },
            status=>{
                let err= res.text().await
                    .map(|body| Error::Custom(format!("Server returned unexpected status code {} and body {}", status, body)))?;
                Err(err)
            }
        }
    }
    ///adds CheckRedirect to the client the reassign new http client to the Client
    pub fn add_check_redirect<T>(&mut self, policy: T) -> Result<()>
    where
        T: Fn(Attempt) -> Action + Send + Sync + 'static,
    {
        let policy = reqwest::redirect::Policy::custom(policy);
        let client = reqwest::Client::builder().redirect(policy).build()?;
        self.http_client = client;
        Ok(())
    }
}

#[cfg(test)]
mod test_describe {
    use crate::client::Client;
    use crate::utils::FUNCTION_PATH;
    use reqwest::StatusCode;
    use utility::faas::types::model::FunctionStatus;

    fn make_expected_get_func_info_response() -> String {
        let status = FunctionStatus {
            name: "func-test1".into(),
            image: "image-test1".into(),
            replicas: 1,
            available_replicas: 0,
            invocation_count: 1.0,
            env_process: "env-process test1".into(),
            env_vars: Default::default(),
            constraints: vec![],
            secrets: vec![],
            labels: Default::default(),
            annotations: Default::default(),
            limits: Default::default(),
            requests: Default::default(),
            namespace: "".to_string(),
            read_only_root_filesystem: false,
            created_at: "".to_string(),
        };
        let data = serde_json::to_string(&status).unwrap();
        data
    }

    #[tokio::test]
    async fn test_get_func_info() {
        let expected = make_expected_get_func_info_response();
        let _mok = mockito::mock("GET", format!("{}/func-test1", FUNCTION_PATH).as_str())
            .with_status(200)
            .with_body(expected.clone())
            .create();
        let cli_auth = crate::TestAuth {};

        let add = format!("http://{}", mockito::server_address().to_string());
        let client = Client::new(Box::new(&cli_auth), add.as_str()).unwrap();

        let res = client.get_function_info("func-test1", "").await;
        assert!(res.is_ok());
        let res = res.unwrap();
        let out = serde_json::to_string(&res).unwrap();
        assert_eq!(out, expected);
    }

    #[tokio::test]
    async fn test_get_func_info_not_ok() {
        let _mok = mockito::mock("GET", format!("{}/func-test1", FUNCTION_PATH).as_str())
            .with_status(StatusCode::BAD_REQUEST.as_u16() as usize)
            .create();
        let cli_auth = crate::TestAuth {};
        let add = format!("http://{}", mockito::server_address().to_string());
        let client = Client::new(Box::new(&cli_auth), add.as_str()).unwrap();
        let res = client.get_function_info("func-test1", "").await;
        assert!(res.is_err());
        assert!(format!("{:?}", res).contains("Server returned unexpected status code"));
    }

    #[tokio::test]
    async fn test_get_func_info_not_found() {
        let _mok = mockito::mock("GET", format!("{}/func-test1", FUNCTION_PATH).as_str())
            .with_status(StatusCode::NOT_FOUND.as_u16() as usize)
            .create();
        let cli_auth = crate::TestAuth {};
        let add = format!("http://{}", mockito::server_address().to_string());
        let client = Client::new(Box::new(&cli_auth), add.as_str()).unwrap();
        let res = client.get_function_info("func-test1", "").await;
        assert!(res.is_err());
        let expected = format!("No such function: func-test1");
        assert!(format!("{:?}", res).contains(expected.as_str()));
    }
}
