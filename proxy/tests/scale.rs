use proxy::client::Client;
use proxy::utils::SCALE_PATH;
use proxy::TestAuth;
use reqwest::StatusCode;

#[tokio::test]
async fn test_scale_function() {
    let add = format!("http://{}", mockito::server_address().to_string());
    let func = "function-to-scale";
    let path = format!("{}/{}", SCALE_PATH, func);
    let _mok = mockito::mock("POST", path.as_str())
        .with_status(200)
        .create();
    let auth = TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str()).unwrap();
    let res = client.scale_function(func, "", 0).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_scale_function_404() {
    let add = format!("http://{}", mockito::server_address().to_string());
    let func = "function-to-scale";
    let path = format!("{}/{}", SCALE_PATH, func);
    let _mok = mockito::mock("POST", path.as_str())
        .with_status(StatusCode::NOT_FOUND.as_u16() as usize)
        .create();
    let auth = TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str()).unwrap();
    let res = client.scale_function(func, "", 0).await;
    assert!(res.is_err());
    assert!(format!("{:?}", res).contains(format!("No such function: {}", func).as_str()));
}

#[tokio::test]
async fn test_scale_function_unauthorized() {
    let add = format!("http://{}", mockito::server_address().to_string());
    let func = "function-to-scale";
    let path = format!("{}/{}", SCALE_PATH, func);
    let _mok = mockito::mock("POST", path.as_str())
        .with_status(StatusCode::UNAUTHORIZED.as_u16() as usize)
        .create();
    let auth = TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str()).unwrap();
    let res = client.scale_function(func, "", 0).await;
    assert!(res.is_err());
    assert!(format!("{:?}", res).contains("unauthorized access"));
}

#[tokio::test]
async fn test_scale_function_not2xx_and404() {
    let add = format!("http://{}", mockito::server_address().to_string());
    let func = "function-to-scale";
    let path = format!("{}/{}", SCALE_PATH, func);
    let _mok = mockito::mock("POST", path.as_str())
        .with_status(StatusCode::BAD_REQUEST.as_u16() as usize)
        .create();
    let auth = TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str()).unwrap();
    let res = client.scale_function(func, "", 0).await;
    assert!(res.is_err());
    assert!(format!("{:?}", res).contains("Server returned unexpected status code"));
}
