use mockito::Matcher;
use proxy::client::Client;
use proxy::utils::{NAMESPACE_KEY, SECRET_ENDPOINT};
use reqwest::StatusCode;
use utility::faas::types::model::Secret;

#[tokio::test]
async fn test_get_secret_200() {
    let add = format!("http://{}", mockito::server_address().to_string());
    let namespace = "openfaas-fn";
    let _mok = mockito::mock("GET", SECRET_ENDPOINT)
        .match_query(Matcher::UrlEncoded(NAMESPACE_KEY.into(), namespace.into()))
        .with_status(200)
        //.with_header("Authorization", "Bearer abc123")
        .with_body(r#"[]"#)
        .create();

    let auth = proxy::TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.get_secret_list(namespace).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_get_secret_202_accepted() {
    let add = format!("http://{}", mockito::server_address().to_string());
    let namespace = "openfaas-fn";
    let _mok = mockito::mock("GET", SECRET_ENDPOINT)
        .match_query(Matcher::UrlEncoded(NAMESPACE_KEY.into(), namespace.into()))
        .with_status(202)
        //.with_header("Authorization", "Bearer abc123")
        .with_body(r#"[]"#)
        .create();

    let auth = proxy::TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.get_secret_list(namespace).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_get_secret_bad_request() {
    let add = format!("http://{}", mockito::server_address().to_string());
    let namespace = "openfaas-fn";
    let _mok = mockito::mock("GET", SECRET_ENDPOINT)
        .match_query(Matcher::UrlEncoded(NAMESPACE_KEY.into(), namespace.into()))
        .with_status(StatusCode::BAD_REQUEST.as_u16() as usize)
        //.with_header("Authorization", "Bearer abc123")
        .with_body(r#"[]"#)
        .create();

    let auth = proxy::TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.get_secret_list(namespace).await;
    assert!(res.is_err());
    assert!(format!("{:?}", res).contains("Server returned unexpected status code"));
}

#[tokio::test]
async fn test_get_secret_unauthorized() {
    let add = format!("http://{}", mockito::server_address().to_string());
    let namespace = "openfaas-fn";
    let _mok = mockito::mock("GET", SECRET_ENDPOINT)
        .match_query(Matcher::UrlEncoded(NAMESPACE_KEY.into(), namespace.into()))
        .with_status(StatusCode::UNAUTHORIZED.as_u16() as usize)
        //.with_header("Authorization", "Bearer abc123")
        .with_body(r#"[]"#)
        .create();

    let auth = proxy::TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.get_secret_list(namespace).await;
    assert!(res.is_err());
    assert!(format!("{:?}", res).contains("unauthorized access"));
}

#[tokio::test]
async fn test_create_secret_ok200() {
    // let expected = vec![
    //     Secret{
    //         name: "Secret1".into(),..Default::default()
    //     },
    //     Secret{
    //         name: "Secret2".into(),..Default::default()
    //     },
    // ];

    let secret = Secret {
        name: "secret-name".into(),
        value: "secret-value".into(),
        namespace: "openfaas-fn".into(),
    };
    let add = format!("http://{}", mockito::server_address().to_string());
    let _mok = mockito::mock("POST", SECRET_ENDPOINT)
        //.match_query(Matcher::UrlEncoded(NAMESPACE_KEY.into(), namespace.into()))
        .with_status(StatusCode::OK.as_u16() as usize)
        //.with_header("Authorization", "Bearer abc123")
        .with_body(r#"[]"#)
        .create();

    let auth = proxy::TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.create_secret(&secret).await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), (200u16, "Created".to_string()));
}

#[tokio::test]
async fn test_create_secret_created201() {
    // let expected = vec![
    //     Secret{
    //         name: "Secret1".into(),..Default::default()
    //     },
    //     Secret{
    //         name: "Secret2".into(),..Default::default()
    //     },
    // ];

    let secret = Secret {
        name: "secret-name".into(),
        value: "secret-value".into(),
        namespace: "openfaas-fn".into(),
    };
    let add = format!("http://{}", mockito::server_address().to_string());
    let _mok = mockito::mock("POST", SECRET_ENDPOINT)
        //.match_query(Matcher::UrlEncoded(NAMESPACE_KEY.into(), namespace.into()))
        .with_status(StatusCode::CREATED.as_u16() as usize)
        //.with_header("Authorization", "Bearer abc123")
        .with_body(r#"[]"#)
        .create();

    let auth = proxy::TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.create_secret(&secret).await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), (201u16, "Created".to_string()));
}

#[tokio::test]
async fn test_create_secret_accepted202() {
    // let expected = vec![
    //     Secret{
    //         name: "Secret1".into(),..Default::default()
    //     },
    //     Secret{
    //         name: "Secret2".into(),..Default::default()
    //     },
    // ];

    let secret = Secret {
        name: "secret-name".into(),
        value: "secret-value".into(),
        namespace: "openfaas-fn".into(),
    };
    let add = format!("http://{}", mockito::server_address().to_string());
    let _mok = mockito::mock("POST", SECRET_ENDPOINT)
        //.match_query(Matcher::UrlEncoded(NAMESPACE_KEY.into(), namespace.into()))
        .with_status(StatusCode::ACCEPTED.as_u16() as usize)
        //.with_header("Authorization", "Bearer abc123")
        .with_body(r#"[]"#)
        .create();

    let auth = proxy::TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.create_secret(&secret).await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), (202u16, "Created".to_string()));
}

#[tokio::test]
async fn test_create_secret_bad_request() {
    // let expected = vec![
    //     Secret{
    //         name: "Secret1".into(),..Default::default()
    //     },
    //     Secret{
    //         name: "Secret2".into(),..Default::default()
    //     },
    // ];

    let secret = Secret {
        name: "secret-name".into(),
        value: "secret-value".into(),
        namespace: "openfaas-fn".into(),
    };
    let add = format!("http://{}", mockito::server_address().to_string());
    let _mok = mockito::mock("POST", SECRET_ENDPOINT)
        //.match_query(Matcher::UrlEncoded(NAMESPACE_KEY.into(), namespace.into()))
        .with_status(StatusCode::BAD_REQUEST.as_u16() as usize)
        //.with_header("Authorization", "Bearer abc123")
        .with_body(r#"[]"#)
        .create();

    let auth = proxy::TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.create_secret(&secret).await;
    assert!(res.is_ok());
    assert!(format!("{:?}", res).contains("Server returned unexpected status code"));
}

#[tokio::test]
async fn test_create_secret_unauthorized() {
    // let expected = vec![
    //     Secret{
    //         name: "Secret1".into(),..Default::default()
    //     },
    //     Secret{
    //         name: "Secret2".into(),..Default::default()
    //     },
    // ];

    let secret = Secret {
        name: "secret-name".into(),
        value: "secret-value".into(),
        namespace: "openfaas-fn".into(),
    };
    let add = format!("http://{}", mockito::server_address().to_string());
    let _mok = mockito::mock("POST", SECRET_ENDPOINT)
        //.match_query(Matcher::UrlEncoded(NAMESPACE_KEY.into(), namespace.into()))
        .with_status(StatusCode::UNAUTHORIZED.as_u16() as usize)
        //.with_header("Authorization", "Bearer abc123")
        .with_body(r#"[]"#)
        .create();

    let auth = proxy::TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.create_secret(&secret).await;
    assert!(res.is_ok());
    assert!(format!("{:?}", res).contains("unauthorized access"));
}

#[tokio::test]
async fn test_create_secret_conflict() {
    // let expected = vec![
    //     Secret{
    //         name: "Secret1".into(),..Default::default()
    //     },
    //     Secret{
    //         name: "Secret2".into(),..Default::default()
    //     },
    // ];

    let secret = Secret {
        name: "secret-name".into(),
        value: "secret-value".into(),
        namespace: "openfaas-fn".into(),
    };
    let add = format!("http://{}", mockito::server_address().to_string());
    let _mok = mockito::mock("POST", SECRET_ENDPOINT)
        //.match_query(Matcher::UrlEncoded(NAMESPACE_KEY.into(), namespace.into()))
        .with_status(StatusCode::CONFLICT.as_u16() as usize)
        //.with_header("Authorization", "Bearer abc123")
        .with_body(r#"[]"#)
        .create();

    let auth = proxy::TestAuth {};
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.create_secret(&secret).await;
    assert!(res.is_ok());
    assert!(format!("{:?}", res).contains("already exists"));
}
