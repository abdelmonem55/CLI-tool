use proxy::function_store::function_store_list;
use schema::store::v2::store::StoreFunction;
use std::collections::HashMap;

const TEST_STACK: &str = r#"
{
    "version": "0.2.0",
    "functions": [
    {
        "title": "NodeInfo",
        "name": "nodeinfo",
        "description": "Get info about the machine that you're deployed on. Tells CPU count, hostname, OS, and Uptime",
        "images": {
            "arm64": "functions/nodeinfo:arm64",
            "armhf": "functions/nodeinfo-http:latest-armhf",
            "x86_64": "functions/nodeinfo-http:latest"
        },
        "repo_url": "https://github.com/openfaas/faas/tree/master/sample-functions/NodeInfo"
    }]
}
"#;

#[tokio::test]
async fn test_generate() {
    let _mok = mockito::mock("GET", "/functions")
        .with_status(200)
        .with_body(TEST_STACK)
        .create();

    let want =vec![StoreFunction{
        icon: "".to_string(),
        title:                  "NodeInfo".into(),
        name:                   "nodeinfo".into(),
        fprocess: "".to_string(),
        description:            "Get info about the machine that you're deployed on. Tells CPU count, hostname, OS, and Uptime".into(),
        images:                 [("arm64".to_string(),"functions/nodeinfo:arm64".to_string())
                                , ("armhf".to_string(), "functions/nodeinfo-http:latest-armhf".to_string())
                                , ("x86_64".to_string(),"functions/nodeinfo-http:latest".to_string() )].iter().cloned().collect(),
        repo_url:                "https://github.com/openfaas/faas/tree/master/sample-functions/NodeInfo".into(),
        read_only_root_filesystem: false,
        environment:            HashMap::new(),
        labels:                 HashMap::new(),
        annotations:            HashMap::new(),
        network: "".to_string()
    }];
    let add = mockito::server_address().to_string();
    let addr = format!("http://{}/functions", add);
    let list = function_store_list(addr.as_str()).await;
    assert!(list.is_ok());
    let list = list.unwrap();
    assert_eq!(list, want);
}
