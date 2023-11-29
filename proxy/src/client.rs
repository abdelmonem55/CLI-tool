use reqwest::Url;
use std::collections::HashMap;
use url::ParseError;
use utility::{Error, Result};

///an API client to perform all operations
pub struct Client<'s> {
    pub(crate) http_client: reqwest::Client,
    ///ClientAuth a type implementing ClientAuth interface for client authentication
    pub client_auth: Box<&'s dyn ClientAuth>,
    ///Gatewayurl base url of OpenFaaS gateway
    pub gateway: Url,
    ///user agent for the client
    pub user_agent: Option<&'s str>,
}

///ClientAuth an interface for client authentication.
// to add authentication to the client implement this interface
pub trait ClientAuth {
    fn set(&self, req: &mut reqwest::RequestBuilder) -> Result<()>;
}

impl<'s> Client<'s> {
    /// initializes a new API client
    pub fn new(auth: Box<&'s dyn ClientAuth>, gateway: &str) -> Result<Client<'s>> {
        let url = gateway.trim_end_matches('/');
        let url = Url::parse(url).map_err(|e| Error::Custom(format!("{:?}", e)))?;

        Ok(Client {
            http_client: reqwest::Client::new(),
            client_auth: auth,
            gateway: url,
            user_agent: None,
        })
    }

    ///create a new HTTP request with authentication
    pub fn new_request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: String,
    ) -> Result<reqwest::RequestBuilder> {
        let u = match url::Url::parse(path) {
            Ok(u) => u,
            Err(e) => {
                if e == ParseError::RelativeUrlWithoutBase {
                    let url = format!("http://example.com/{}", path.trim_start_matches("/"));
                    url::Url::parse(url.as_str())?
                } else {
                    return Err(Error::Custom(format!("{:?}", e)));
                }
            }
        };

        // deep copy gateway url and then add the supplied path  and args to the copy so that
        // we preserve the original gateway url as much as possible
        let mut endpoint = self.gateway.clone();
        let path = std::path::Path::new(endpoint.path()).join(u.path().trim_start_matches("/"));
        endpoint.set_path(path.to_string_lossy().to_string().as_str());
        endpoint.set_query(u.query());
        // let req = http::Request::builder()
        //     .method(method)
        //     .uri(endpoint.into_string());
        //
        // let req =if !body.is_empty() {
        //     req.header("Content-Type", "application/json").body(body)
        // }else {
        //     req.body("")
        // };
        //
        // let req = req.map_err(|e| Error::IoCustom(format!("{:?}",e)))?;

        let mut req = self.http_client.request(method, endpoint);
        // reqwest::redirect::Policy::custom()

        if !body.is_empty() {
            req = req.header("Content-Type", "application/json").body(body);
        } else {
            req = req.body("");
        }
        //let req = req.build()?;
        self.client_auth.set(&mut req)?;

        Ok(req)
    }

    ///perform an HTTP request with context
    pub(crate) async fn do_request(&self, req: reqwest::Request) -> Result<reqwest::Response> {
        // req = req.WithContext(ctx)

        // if val, ok := os.LookupEnv("OPENFAAS_DUMP_HTTP"); ok && val == "true" {
        //     dump, err := httputil.DumpRequest(req, true)
        //     if err != nil {
        //         return nil, err
        //     }
        //     fmt.Println(string(dump))
        // }

        let resp = self.http_client.execute(req).await?;
        Ok(resp)
    }
}

pub(crate) fn add_query_params(url: &str, params: &HashMap<&str, &str>) -> Result<String> {
    let mut relative = false;
    let mut parsed_url = match url::Url::parse(url) {
        Ok(u) => u,
        Err(e) => {
            if e == ParseError::RelativeUrlWithoutBase {
                let url = format!("http://example.com/{}", url.trim_start_matches("/"));
                relative = true;
                url::Url::parse(url.as_str())?
            } else {
                return Err(Error::Custom(format!("{:?}", e)));
            }
        }
    };
    //.map_err(|e| Error::IoCustom(format!("{:?}",e)))?;
    {
        let mut modifier = parsed_url.query_pairs_mut();
        for (key, val) in params {
            modifier.append_pair(key, val);
        }
    }

    let url = parsed_url.to_string();
    if relative {
        Ok(url.trim_start_matches("http://example.com").to_string())
    } else {
        Ok(url)
    }
}

#[test]
fn test_add_query_params() {
    struct TestCase {
        _name: &'static str,
        params: HashMap<&'static str, &'static str>,
        url: &'static str,
        expected_url: &'static str,
    }

    let cases = vec![
        TestCase {
            _name: "url without hostname",
            params: [("namespace", "openfaas-fn")].iter().cloned().collect(),
            url: "/system/functions",
            expected_url: "/system/functions?namespace=openfaas-fn",
        },
        TestCase {
            _name: "url hostname",
            params: [("namespace", "openfaas-fn")].iter().cloned().collect(),
            url: "http://127.0.0.1/system/functions",
            expected_url: "http://127.0.0.1/system/functions?namespace=openfaas-fn",
        },
        TestCase {
            _name: "A url with simple hostname",
            params: [("namespace", "openfaas-fn")].iter().cloned().collect(),
            url: "example",
            expected_url: "/example?namespace=openfaas-fn",
        },
    ];

    for test in cases {
        let res = add_query_params(test.url, &test.params);
        assert!(res.is_ok());
        let url = res.unwrap();
        assert_eq!(url, test.expected_url);
    }
}
