use lazy_static::lazy_static;
use proxy::client::Client;
use proxy::utils::SYSTEM_PATH;
use proxy::TestAuth;
use utility::faas::types::model::FunctionStatus;

lazy_static! {
    static ref LIST_FUNCTION_RESP: Vec<FunctionStatus> = vec![
        FunctionStatus {
            name: String::from("func-test1"),
            image: "image-test1".to_string(),
            namespace: "".to_string(),
            env_vars: Default::default(),
            constraints: vec![],
            secrets: vec![],
            labels: Default::default(),
            annotations: Default::default(),
            limits: Default::default(),
            requests: Default::default(),
            read_only_root_filesystem: false,
            replicas: 1,
            available_replicas: 0,
            invocation_count: 1.,
            env_process: "env-process test1".into(),
            created_at: "".to_string()
        },
        FunctionStatus {
            name: "func-test2".into(),
            image: "image-test2".into(),
            replicas: 2,
            available_replicas: 0,
            invocation_count: 2.,
            env_process: "env-process test2".into(),
            env_vars: Default::default(),
            constraints: vec![],
            secrets: vec![],
            labels: Default::default(),
            annotations: Default::default(),
            limits: Default::default(),
            requests: Default::default(),
            namespace: "".to_string(),
            read_only_root_filesystem: false,
            created_at: "".to_string()
        }
    ];
}

#[tokio::test]
async fn test_list_functions_ok() {
    let data = LIST_FUNCTION_RESP.clone();
    let body = serde_json::to_string(&data).unwrap();
    println!("{}", body);
    let _mok = mockito::mock("GET", SYSTEM_PATH)
        .with_status(200)
        .with_body(body)
        .create();
    let cli_auth = TestAuth {};

    let add = format!("http://{}", mockito::server_address().to_string());
    let mut client = Client::new(Box::new(&cli_auth), add.as_str()).unwrap();
    let res = client.list_functions("").await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), data);
}

#[tokio::test]
async fn test_list_functions_not200() {
    let data = LIST_FUNCTION_RESP.clone();
    let body = serde_json::to_string(&data).unwrap();
    let _mok = mockito::mock("GET", SYSTEM_PATH)
        .with_status(400)
        .with_body(body)
        .create();
    let cli_auth = TestAuth {};

    let add = format!("http://{}", mockito::server_address().to_string());
    let mut client = Client::new(Box::new(&cli_auth), add.as_str()).unwrap();
    let res = client.list_functions("").await;
    assert!(res.is_err());
    assert!(format!("{:?}", res).contains("Server returned unexpected status code"));
}
