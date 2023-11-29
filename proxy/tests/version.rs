use proxy::client::Client;
use proxy::utils::INFO_ENDPOINT;

#[tokio::test]
async fn test_get_system_info_ok() {
    let add = format!("http://{}", mockito::server_address().to_string());
    let _mok = mockito::mock("GET", INFO_ENDPOINT)
        .with_status(200)
        .with_body("{}")
        .create();

    let auth = proxy::TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.get_system_info().await;
    assert!(res.is_ok());
}
