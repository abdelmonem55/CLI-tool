use proxy::client::*;
use utility::Result;

#[derive(Clone)]
struct TestAuth;

impl ClientAuth for TestAuth {
    fn set(&self, _req: &mut reqwest::RequestBuilder) -> Result<()> {
        Ok(())
    }
}
#[test]
fn test_new_client() {
    let auth = TestAuth {};
    struct TestCase {
        _name: &'static str,
        input: &'static str,
        output: &'static str,
    }

    let cases = vec![
        TestCase {
            _name: "Without trailing slash",
            input: "http://127.0.0.1:8080",
            output: "http://127.0.0.1:8080/",
        },
        TestCase {
            _name: "With trailing slash",
            input: "http://127.0.0.1:8080/",
            output: "http://127.0.0.1:8080/",
        },
    ];

    for test in cases {
        let auth = auth.clone();
        let res = Client::new(Box::new(&auth), test.input);
        assert!(res.is_ok());
        let client = res.unwrap();
        let url = client.gateway.to_string();
        assert_eq!(test.output, url);
    }
}

#[test]
fn test_new_request_url() {
    let auth = TestAuth {};
    let gateway = "http://127.0.0.1:8080/base/path";
    let client = Client::new(Box::new(&auth), gateway).unwrap();

    struct TestCase {
        _name: &'static str,
        path: &'static str,
        expected_url: &'static str,
    }
    let cases = vec![
        TestCase {
            _name: "A valid path",
            path: "http://127.0.0.1:8080/system/functions",
            expected_url: "http://127.0.0.1:8080/base/path/system/functions",
        },
        TestCase {
            _name: "Root path",
            path: "http://127.0.0.1:8080/",
            expected_url: "http://127.0.0.1:8080/base/path/",
        },
        TestCase {
            _name: "path without starting slash",
            path: "http://127.0.0.1:8080/system/functions",
            expected_url: "http://127.0.0.1:8080/base/path/system/functions",
        },
        TestCase {
            _name: "path with querystring",
            path: "http://127.0.0.1:8080/system/functions?namespace=fn",
            expected_url: "http://127.0.0.1:8080/base/path/system/functions?namespace=fn",
        },
    ];

    for test in cases {
        let req = client.new_request(reqwest::Method::POST, test.path, "".into());
        assert!(req.is_ok());
        let req = req.unwrap().build();
        assert!(req.is_ok());
        let req = req.unwrap();
        assert_eq!(req.url().as_str(), test.expected_url)
    }
}
