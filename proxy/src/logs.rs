use crate::client::Client;
use crate::utils::LOGS_PATH;
use reqwest::{Method, StatusCode};
use std::collections::HashMap;
use utility::faas_provider::logs::{Message, Request};
use utility::{Error, Result};

impl<'s> Client<'s> {
    ///return stream for the logs
    pub async fn get_logs(&self, params: Request<'s>) -> Result<Vec<Message>> {
        let url = format!(
            "{}/{}",
            self.gateway.as_str().trim_end_matches('/'),
            LOGS_PATH.trim_start_matches("/")
        );
        let mut url = url::Url::parse(url.as_str())?;
        {
            let mut url_query = url.query_pairs_mut();
            let params = req_as_query_values(&params);
            url_query.extend_pairs(params);
        }
        let log_request = self
            .new_request(Method::GET, url.as_str(), "".into())
            .map_err(|e| {
                Error::Custom(format!(
                    "can't read logs from OpenFaaS on URL {}\nand debug reason {:?}",
                    self.gateway.as_str(),
                    e
                ))
            })?
            .build()
            .map_err(|e| {
                Error::Custom(format!(
                    "can't read logs from OpenFaaS on URL {}\nand debug reason {:?}",
                    self.gateway.as_str(),
                    e
                ))
            })?;

        let res = self.do_request(log_request).await?;
        match res.status() {
            StatusCode::OK => {
                let body = res.text().await?;
                if !body.is_empty() {
                    let status: Vec<Message> = serde_json::from_str(body.as_str())
                        .map_err(|e| Error::Custom( format!("can't read logs from OpenFaaS on URL {}\nand debug reason {:?}", self.gateway.as_str(), e)))?;
                    Ok(status)
                }else {
                    Ok(vec![])
                }
            },
            StatusCode::UNAUTHORIZED => {
                Err(Error::Custom(format!("unauthorized access, run \"faas-cli login\" to setup authentication for this server")))
            },
            status => {
                let err = res.text().await
                    .map(|body| Error::Custom(format!("Server returned unexpected status code {} and body {}", status, body)))?;
                Err(err)
            }
        }
    }
}

fn req_as_query_values(r: &Request) -> HashMap<String, String> {
    let mut query = HashMap::new();
    query.insert("name".to_string(), r.name.to_string());
    if !r.namespace.is_empty() {
        query.insert("namespace".to_string(), r.namespace.to_string());
    }
    query.insert("follow".to_string(), r.follow.to_string());
    if !r.instance.is_empty() {
        query.insert("instance".to_string(), r.instance.to_string());
    }

    if r.since.is_some() {
        query.insert("since".to_string(), r.since.unwrap().to_string());
    }

    if r.tail != 0 {
        query.insert("tail".to_string(), r.tail.to_string());
    }

    return query;
}

// fn makeStreamingHTTPClient(tlsInsecure bool) /*http.Client*/ {
//     client := http.Client{}
//
//     if tlsInsecure {
//         tr := &http.Transport{
//             Proxy: http.ProxyFromEnvironment,
//         }
//
//         if tlsInsecure {
//             tr.TLSClientConfig = &tls.Config{InsecureSkipVerify: tlsInsecure}
//         }
//
//         client.Transport = tr
//     }
//
//     return client
// }
