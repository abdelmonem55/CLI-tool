use crate::proxy::make_http_client;
use reqwest::{Method, StatusCode, Url};
use std::collections::HashMap;
use std::io::Write;
use utility::{Error, Result};

/// InvokeFunction a function
pub async fn invoke_function(
    gateway: &str,
    name: &str,
    bytes_in: &[u8],
    content_type: &str,
    query: &Vec<&str>,
    headers: &Vec<&str>,
    is_async: bool,
    http_method: Method,
    tls_insecure: bool,
    namespace: &str,
) -> Result<String> {
    let gateway = gateway.trim_end_matches('/');

    let disable_function_timeout = None;
    let client = make_http_client(disable_function_timeout, tls_insecure)?;

    let qs = build_query_string(query)?;

    let header_map = parse_headers(headers)?;

    let function_endpoint = if is_async {
        "/async-function/".to_string()
    } else {
        "/function/".to_string()
    };
    validate_http_method(http_method.clone())?;
    let mut gateway_url = format!("{}{}{}", gateway, function_endpoint, name);

    if !namespace.is_empty() {
        gateway_url.push('.');
        gateway_url.push_str(namespace);
    }
    gateway_url.push_str(qs.as_str());
    let url = Url::parse(gateway_url.as_str()).map_err(|e| Error::Custom(e.to_string()))?;

    let mut req = client
        .request(http_method.clone(), url)
        .header("Content-type", content_type);
    if !bytes_in.is_empty() {
        req = req.body(bytes_in.to_owned())
    }

    // Add additional headers to request
    for (name, value) in &header_map {
        req = req.header(name, value);
    }
    let req = req.build()?;

    // Removed by AE - the system-level basic auth secrets should not be transmitted
    // to functions. Functions should implement their own auth.
    // SetAuth(req, gateway)

    let res = client
        .execute(req)
        .await
        .map_err(|_e| Error::Custom(format!("cannot connect to OpenFaaS on URL: {}", gateway)))?;

    match res.status() {
        StatusCode::ACCEPTED => {
            std::io::stderr()
                .write(b"Function submitted asynchronously.\n")
                .map_err(|e| Error::Custom(e.to_string()))?;
            Ok(String::new())
        }
        StatusCode::OK => {
            let body = res.text().await.map_err(|e| {
                Error::Custom(format!(
                    "cannot read result from OpenFaaS on URL: {} {}",
                    gateway, e
                ))
            })?;
            Ok(body)
        }
        StatusCode::UNAUTHORIZED => Err(Error::Custom(format!(
            "unauthorized access, run \"faas-cli login\" to setup authentication for this server"
        ))),
        status => {
            let err = res.text().await.map(|body| {
                Error::Custom(format!(
                    "Server returned unexpected status code {} and body {}",
                    status, body
                ))
            })?;
            Err(err)
        }
    }
}

fn build_query_string(query: &Vec<&str>) -> Result<String> {
    if !query.is_empty() {
        let mut qs = "?".to_string();
        for q in query {
            qs.push_str(q);
            qs.push('&');
            if !q.contains('=') {
                return Err(Error::Custom(
                    "the --query flags must take the form of key=value (= not found)".to_string(),
                ));
            }
            if q.ends_with('=') {
                return Err(Error::Custom(
                    "the --query flags must take the form of key=value (= not found)".to_string(),
                ));
            }
        }
        qs = qs.trim_end_matches('&').to_string();
        Ok(qs)
    } else {
        Ok(String::new())
    }
}

fn parse_headers(headers: &Vec<&str>) -> Result<HashMap<String, String>> {
    let mut header_map: HashMap<String, String> = HashMap::new();

    for header in headers {
        let header_values = header
            .splitn(2, '=')
            .map(|str| str.to_string())
            .collect::<Vec<String>>();
        if header_values.len() != 2 || header_values[0].is_empty() || header_values[1].is_empty() {
            return Err(Error::Custom(
                "the --header or -H flag must take the form of key=value".to_string(),
            ));
        }
        header_map.insert(header_values[0].clone(), header_values[1].clone());
    }

    Ok(header_map)
}

/// validateMethod validates the HTTP request method
fn validate_http_method(http_method: Method) -> Result<()> {
    let allowed_methods = vec![
        Method::GET,
        Method::POST,
        Method::POST,
        Method::PATCH,
        Method::DELETE,
    ];
    if !allowed_methods.contains(&http_method) {
        Err(Error::Custom(format!(
            "the --method or -m flag must take one of these values ({:?})",
            allowed_methods
        )))
    } else {
        Ok(())
    }
}
