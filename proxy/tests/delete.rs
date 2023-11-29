use proxy::client::Client;
use proxy::utils::SYSTEM_PATH;

#[tokio::test]
async fn test_delete_function_200() {
    let _mok = mockito::mock("DELETE", SYSTEM_PATH)
        .with_status(200)
        //  .with_body(TEST_STACK)
        .create();

    // let req = reqwest::Request::new(Method::DELETE,Url::parse("http://127.0.0.1:1234").unwrap());
    // let cl=reqwest::Client::new();
    // let res =cl.execute(req).await;
    // println!("{:?}",res);

    let cli_auth = proxy::TestAuth {};

    let add = format!("http://{}", mockito::server_address().to_string());
    let client = Client::new(Box::new(&cli_auth), add.as_str()).unwrap();

    let res = client.delete_function("function-to-delete", "").await;

    assert!(res.is_ok());
}

#[tokio::test]
async fn test_delete_function_404() {
    let _mok = mockito::mock("DELETE", SYSTEM_PATH)
        .with_status(404)
        //  .with_body(TEST_STACK)
        .create();
    let cli_auth = proxy::TestAuth {};

    let add = format!("http://{}", mockito::server_address().to_string());
    let client = Client::new(Box::new(&cli_auth), add.as_str()).unwrap();

    let res = client.delete_function("function-to-delete", "").await;

    assert!(format!("{:?}", res).contains("No existing function to remove"));
}

#[tokio::test]
async fn test_delete_function_not404_and_not2xx() {
    let _mok = mockito::mock("DELETE", SYSTEM_PATH)
        .with_status(417)
        //  .with_body(TEST_STACK)
        .create();
    let cli_auth = proxy::TestAuth {};

    let add = format!("http://{}", mockito::server_address().to_string());
    let client = Client::new(Box::new(&cli_auth), add.as_str()).unwrap();

    let res = client.delete_function("function-to-delete", "").await;

    assert!(format!("{:?}", res).contains("Server returned unexpected status code"));
}
