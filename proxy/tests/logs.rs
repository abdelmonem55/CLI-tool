use mockito::Matcher;
use proxy::client::{Client, ClientAuth};
use proxy::utils::LOGS_PATH;
use reqwest::{RequestBuilder, StatusCode};
use utility::faas_provider::logs::{Message, Request};
use utility::{Error, Result};

struct TestTokenAuth<'s> {
    pub token: &'s str,
}

impl<'s> ClientAuth for TestTokenAuth<'s> {
    fn set(&self, req: &mut RequestBuilder) -> Result<()> {
        let req2 = req
            .try_clone()
            .ok_or(Error::Custom(format!("can't clone request :{:?}", req)))?;
        *req = req2.header("Authorization", format!("Bearer {}", self.token));
        Ok(())
    }
}

#[tokio::test]
async fn test_get_logs_auth_log() {
    let expected_token = "abc123";
    let params = Request {
        name: "testFunc",
        ..Default::default()
    };

    let add = format!("http://{}", mockito::server_address().to_string());
    // let endpoint = add + LOGS_PATH;
    // let endpoint = url::Url::parse(endpoint.as_str()).unwrap();

    let _mok = mockito::mock("GET", LOGS_PATH)
        .match_query(Matcher::UrlEncoded("name".into(), params.name.into()))
        .with_status(200)
        .with_header("Authorization", "Bearer abc123")
        .with_body(
            r#"[{
        "name" : "func"
        }]"#,
        )
        .create();
    let auth = TestTokenAuth {
        token: expected_token,
    };
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.get_logs(params).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_get_logs_ok() {
    let expected_token = "abc123";
    let expected = vec![
        Message {
            name: "test1".into(),
            ..Default::default()
        },
        Message {
            name: "test2".into(),
            ..Default::default()
        },
    ];
    let expected_out = serde_json::to_string(&expected).unwrap();
    let params = Request {
        name: "testFunc",
        ..Default::default()
    };

    let add = format!("http://{}", mockito::server_address().to_string());
    // let endpoint = add + LOGS_PATH;
    // let endpoint = url::Url::parse(endpoint.as_str()).unwrap();

    let _mok = mockito::mock("GET", LOGS_PATH)
        .match_query(Matcher::UrlEncoded("name".into(), params.name.into()))
        .with_status(200)
        .with_header("Authorization", "Bearer abc123")
        .with_body(expected_out)
        .create();
    let auth = TestTokenAuth {
        token: expected_token,
    };
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.get_logs(params).await;
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), expected);
}

#[tokio::test]
async fn test_get_logs_unauthorized() {
    let expected_token = "abc123";
    let params = Request {
        name: "testFunc",
        ..Default::default()
    };

    let add = format!("http://{}", mockito::server_address().to_string());
    // let endpoint = add + LOGS_PATH;
    // let endpoint = url::Url::parse(endpoint.as_str()).unwrap();

    let _mok = mockito::mock("GET", LOGS_PATH)
        .match_query(Matcher::UrlEncoded("name".into(), params.name.into()))
        .with_status(StatusCode::UNAUTHORIZED.as_u16() as usize)
        .with_body("not allowed")
        .create();
    let auth = TestTokenAuth {
        token: expected_token,
    };
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.get_logs(params).await;
    assert!(res.is_err());
    assert!(format!("{:?}", res).contains("unauthorized access"));
}

#[tokio::test]
async fn test_get_logs_bad_request() {
    let expected_token = "abc123";
    let params = Request {
        name: "testFunc",
        ..Default::default()
    };

    let add = format!("http://{}", mockito::server_address().to_string());
    // let endpoint = add + LOGS_PATH;
    // let endpoint = url::Url::parse(endpoint.as_str()).unwrap();

    let _mok = mockito::mock("GET", LOGS_PATH)
        .match_query(Matcher::UrlEncoded("name".into(), params.name.into()))
        .with_status(StatusCode::BAD_REQUEST.as_u16() as usize)
        .with_body("bad request try again")
        .create();
    let auth = TestTokenAuth {
        token: expected_token,
    };
    let client = Client::new(Box::new(&auth), add.as_str());
    assert!(client.is_ok());
    let client = client.unwrap();
    let res = client.get_logs(params).await;
    assert!(res.is_err());
    assert!(format!("{:?}", res).contains("Server returned unexpected status code"));
}
