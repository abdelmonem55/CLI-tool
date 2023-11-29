use crate::client::Client;
use crate::utils::SYSTEM_PATH;
use reqwest::{Method, StatusCode};
use std::collections::HashMap;
use utility::faas::types::model::{FunctionDeployment, FunctionResources};
use utility::{Error, Result};

///60second
pub const DEFAULT_COMMAND_TIMEOUT: usize = 60;

/// FunctionResourceRequest defines a request to set function resources
#[derive(Default)]
pub struct FunctionResourceRequest {
    pub limits: Option<FunctionResources>,
    pub requests: Option<FunctionResources>,
}

// DeployFunctionSpec defines the spec used when deploying a function
pub struct DeployFunctionSpec {
    pub fprocess: String,
    pub function_name: String,
    pub image: String,
    pub registry_auth: String,
    pub language: String,
    pub replace: bool,
    pub env_vars: HashMap<String, String>,
    pub network: String,
    pub constraints: Vec<String>,
    pub update: bool,
    pub secrets: Vec<String>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    pub function_resource_request: FunctionResourceRequest,
    pub read_only_root_filesystem: bool,
    pub tls_insecure: bool,
    pub token: String,
    pub namespace: String,
}

pub fn generate_func_str(spec: &DeployFunctionSpec) -> String {
    if !spec.namespace.is_empty() {
        format!("{}.{}", spec.function_name, spec.namespace)
    } else {
        spec.function_name.clone()
    }
}

impl<'s> Client<'s> {
    /// DeployFunction first tries to deploy a function and if it exists will then attempt
    /// a rolling update. Warnings are suppressed for the second API call (if required.)
    /// return status code and string for describing
    pub async fn deploy_function(&self, spec: &DeployFunctionSpec) -> Result<(u16, String)> {
        let rolling_update_info = format!(
            "Function {} already exists, attempting rolling-update.",
            spec.function_name
        );
        let (mut status_code, mut deploy_output) = self.deploy(spec, spec.update).await?;

        if spec.update == true && status_code == StatusCode::NOT_FOUND {
            // Re-run the function with update=false

            let res = self.deploy(spec, false).await?;
            status_code = res.0;
            deploy_output = res.1;
        } else if status_code == StatusCode::OK {
            println!("{}", rolling_update_info);
        }
        //println!("\n{}", deploy_output);
        return Ok((status_code, deploy_output));
    }

    /// deploy a function to an OpenFaaS gateway over REST
    pub(crate) async fn deploy(
        &self,
        spec: &DeployFunctionSpec,
        update: bool,
    ) -> Result<(u16, String)> {
        // Need to alter Gateway to allow nil/empty string as fprocess, to avoid this repetition.
        let mut fprocess_template: String = "".into();
        if !spec.fprocess.is_empty() {
            fprocess_template = spec.fprocess.clone();
        }

        if spec.replace {
            self.delete_function(spec.function_name.as_str(), spec.namespace.as_str())
                .await?;
        }

        let mut req = FunctionDeployment {
            env_process: fprocess_template,
            image: spec.image.clone(),
            service: spec.function_name.clone(),
            env_vars: spec.env_vars.clone(),
            constraints: spec.constraints.clone(),
            secrets: spec.secrets.clone(),
            labels: spec.labels.clone(),
            annotations: spec.annotations.clone(),
            limits: None,
            requests: None,
            read_only_root_filesystem: spec.read_only_root_filesystem.clone(),
            namespace: spec.namespace.clone(),
        };

        let mut has_limits = false;
        //req.limits =Some(FunctionResources{ memory: "".to_string(), cpu: "".to_string() });
        if let Some(limit) = &spec.function_resource_request.limits {
            let mut resource = FunctionResources {
                memory: "".to_string(),
                cpu: "".to_string(),
            };
            if !limit.memory.is_empty() {
                resource.memory = limit.memory.clone();
                has_limits = true;
            }
            if !limit.cpu.is_empty() {
                resource.cpu = limit.cpu.clone();
                has_limits = true;
            }
            if !has_limits {
                req.limits = None;
            } else {
                req.limits = Some(resource);
            }
        }

        let mut has_requests = false;
        //req.requests =Some(FunctionResources{ memory: "".to_string(), cpu: "".to_string() });
        if let Some(request) = &spec.function_resource_request.requests {
            let mut resource = FunctionResources {
                memory: "".to_string(),
                cpu: "".to_string(),
            };
            if !request.memory.is_empty() {
                resource.memory = request.memory.clone();
                has_requests = true;
            }
            if !request.cpu.is_empty() {
                resource.cpu = request.cpu.clone();
                has_requests = true;
            }
            if !has_requests {
                req.requests = None;
            } else {
                req.requests = Some(resource);
            }
        }
        let req_bytes = serde_json::to_string(&req).map_err(|e| Error::Custom(format!("{}", e)))?;

        let mut method = Method::POST;
        // "application/json"
        if update {
            method = Method::PUT;
        }

        let request = self
            .new_request(method, SYSTEM_PATH, req_bytes)
            .map_err(|e| Error::Custom(format!("InternalServerError:{}", e)))?;

        let request = request.build()?;

        let response = self
            .do_request(request)
            .await
            .map_err(|e| Error::Custom(format!("InternalServerError:{}", e)))?;
        //println!("{:?}",response.url().as_str());

        let mut need_body = false;

        let resp = &response;
        let res = match resp.status() {
            StatusCode::OK | StatusCode::CREATED | StatusCode::ACCEPTED => {
                let deployed_url = format!("{}/function/{}", self.gateway, generate_func_str(spec));
                let deploy_output = format!("Deployed. {}.\nURL: {}", resp.status(), deployed_url);
                Ok((resp.status().as_u16(), deploy_output))
            }
            StatusCode::UNAUTHORIZED => {
                let deploy_output = format!("unauthorized access, run \"faas-cli login\" to setup authentication for this server");
                Ok((resp.status().as_u16(), deploy_output))
            }
            status => {
                let deploy_output = format!("Unexpected status: {}, message: ", status.as_u16());
                need_body = true;
                Ok((resp.status().as_u16(), deploy_output))
            }
        };
        if need_body {
            let mut res = res.unwrap();
            res.1 = res.1 + response.text().await?.as_str();
            Ok(res)
        } else {
            res
        }
    }
}

#[cfg(test)]
mod test_deploy {
    use crate::client::Client;
    use crate::deploy::{generate_func_str, DeployFunctionSpec, FunctionResourceRequest};
    use crate::utils::SYSTEM_PATH;
    use reqwest::StatusCode;
    use utility::Result;

    const TLS_NO_VERIFY: bool = true;

    struct DeployProxyTest {
        _title: &'static str,
        mock_server_responses: u16,
        replace: bool,
        update: bool,
        method: &'static str,
        expected_output: &'static str,
    }

    #[tokio::test]
    async fn run_deploy_proxy_test1() {
        let dep = DeployProxyTest {
            _title: "",
            mock_server_responses: 200,
            replace: false,
            update: false,
            method: "POST",
            expected_output: "",
        };
        let _mok = mockito::mock(dep.method, SYSTEM_PATH)
            .with_status(dep.mock_server_responses as usize)
            //  .with_body(TEST_STACK)
            .create();
        println!("{:?}", _mok);

        let cli_auth = crate::TestAuth {};
        let add = format!("http://{}", mockito::server_address().to_string());
        let client = Client::new(Box::new(&cli_auth), add.as_str()).unwrap();

        let stdout = client
            .deploy_function(&DeployFunctionSpec {
                fprocess: "fprocess".to_string(),
                function_name: "function".to_string(),
                image: "image".to_string(),
                registry_auth: "dXNlcjpwYXNzd29yZA==".to_string(),
                language: "language".to_string(),
                replace: dep.replace,
                env_vars: Default::default(),
                network: "network".to_string(),
                constraints: vec![],
                update: dep.update,
                secrets: vec![],
                labels: Default::default(),
                annotations: Default::default(),
                function_resource_request: FunctionResourceRequest {
                    limits: None,
                    requests: None,
                },
                read_only_root_filesystem: false,
                tls_insecure: TLS_NO_VERIFY,
                token: "".to_string(),
                namespace: "".to_string(),
            })
            .await;

        println!("{:?}", stdout);
        //   stdout
    }

    async fn run_deploy_proxy_test(dep: &DeployProxyTest) -> Result<(u16, String)> {
        let _mok = mockito::mock(dep.method, SYSTEM_PATH)
            .with_status(dep.mock_server_responses as usize)
            //  .with_body(TEST_STACK)
            .create();
        let _mok = mockito::mock("DELETE", SYSTEM_PATH)
            .with_status(dep.mock_server_responses as usize)
            //  .with_body(TEST_STACK)
            .create();
        //println!("{:?}",_mok);

        let cli_auth = crate::TestAuth {};
        let add = format!("http://{}", mockito::server_address().to_string());
        let client = Client::new(Box::new(&cli_auth), add.as_str()).unwrap();

        let stdout = client
            .deploy_function(&DeployFunctionSpec {
                fprocess: "fprocess".to_string(),
                function_name: "function".to_string(),
                image: "image".to_string(),
                registry_auth: "dXNlcjpwYXNzd29yZA==".to_string(),
                language: "language".to_string(),
                replace: dep.replace,
                env_vars: Default::default(),
                network: "network".to_string(),
                constraints: vec![],
                update: dep.update,
                secrets: vec![],
                labels: Default::default(),
                annotations: Default::default(),
                function_resource_request: FunctionResourceRequest {
                    limits: None,
                    requests: None,
                },
                read_only_root_filesystem: false,
                tls_insecure: TLS_NO_VERIFY,
                token: "".to_string(),
                namespace: "".to_string(),
            })
            .await;

        stdout
    }

    #[tokio::test]
    async fn test_run_deploy_proxy_tests() {
        let cases = vec![
            DeployProxyTest {
                _title: "200_Deploy",
                mock_server_responses: StatusCode::OK.as_u16(),
                replace: true,
                update: false,
                method: "POST",
                expected_output: "Deployed",
            },
            DeployProxyTest {
                method: "POST",
                _title: "404_Deploy",
                mock_server_responses: StatusCode::NOT_FOUND.as_u16(), //StatusCode:: .as_u16(),
                replace: true,
                update: false,
                expected_output: "No existing function to remove",
            },
            DeployProxyTest {
                method: "POST",
                _title: "UpdateFailedDeployed",
                // mockServerResponses: []int{http.StatusNotFound, http.StatusOK},
                mock_server_responses: StatusCode::UNAUTHORIZED.as_u16(),
                replace: false,
                update: false,
                expected_output: "unauthorized access",
            },
        ];
        for (_, case) in cases.iter().enumerate() {
            let res = run_deploy_proxy_test(case).await;
            assert!(format!("{:?}", res).contains(case.expected_output));
        }
    }

    #[test]
    fn test_deploy_function_generate_func_str() {
        struct TestCase {
            _name: &'static str,
            spec: DeployFunctionSpec,
            expected_str: &'static str,
            _should_err: bool,
        }
        let cases = vec![
            TestCase {
                _name: "No Namespace",
                spec: DeployFunctionSpec {
                    fprocess: "fprocess".to_string(),
                    function_name: "funcName".to_string(),
                    image: "image".to_string(),
                    registry_auth: "dXNlcjpwYXNzd29yZA==".to_string(),
                    language: "language".to_string(),
                    replace: false,
                    env_vars: Default::default(),
                    network: "network".to_string(),
                    constraints: vec![],
                    update: false,
                    secrets: vec![],
                    labels: Default::default(),
                    annotations: Default::default(),
                    function_resource_request: FunctionResourceRequest {
                        limits: None,
                        requests: None,
                    },
                    read_only_root_filesystem: false,
                    tls_insecure: TLS_NO_VERIFY,
                    token: "".to_string(),
                    namespace: "".to_string(),
                },
                expected_str: "funcName",
                _should_err: false,
            },
            TestCase {
                _name: "with Namespace",
                spec: DeployFunctionSpec {
                    fprocess: "fprocess".to_string(),
                    function_name: "funcName".to_string(),
                    image: "image".to_string(),
                    registry_auth: "dXNlcjpwYXNzd29yZA==".to_string(),
                    language: "language".to_string(),
                    replace: false,
                    env_vars: Default::default(),
                    network: "network".to_string(),
                    constraints: vec![],
                    update: false,
                    secrets: vec![],
                    labels: Default::default(),
                    annotations: Default::default(),
                    function_resource_request: FunctionResourceRequest {
                        limits: None,
                        requests: None,
                    },
                    read_only_root_filesystem: false,
                    tls_insecure: TLS_NO_VERIFY,
                    token: "".to_string(),
                    namespace: "Namespace".to_string(),
                },
                expected_str: "funcName.Namespace",
                _should_err: false,
            },
        ];

        for test in cases {
            let func_str = generate_func_str(&test.spec);

            assert_eq!(test.expected_str, func_str);
        }
    }
}
