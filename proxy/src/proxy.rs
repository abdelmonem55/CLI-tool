use reqwest::Client;
use std::option::Option::Some;
use std::time::Duration;
use utility::Result;

//makes a HTTP client with good defaults for timeouts.
pub fn make_http_client(timeout: Option<Duration>, _tls_insecure: bool) -> Result<Client> {
    return make_http_client_with_string(timeout, _tls_insecure, false);
}

// make_http_client_with_String makes a HTTP client with good defaults for timeouts.
fn make_http_client_with_string(
    timeout: Option<Duration>,
    tls_insecure: bool,
    _disable_keep_alive: bool,
) -> Result<Client> {
    let mut client = reqwest::ClientBuilder::new();

    if timeout.is_some() || tls_insecure {
        // tr := &http.Transport{
        //     Proxy:             http.ProxyFromEnvironment,
        //     disable_keep_alive: disable_keep_alive,
        // }

        if let Some(time) = timeout {
            client = client.timeout(time);
            // tr.DialContext = (&net.Dialer{
            //     Timeout: *timeout,
            // }).DialContext
            // tr.IdleConnTimeout = 120 * time.Millisecond
            // tr.ExpectContinueTimeout = 1500 * time.Millisecond
        }

        // if tlsInsecure {
        //     tr.TLSClientConfig = &tls.Config{InsecureSkipVerify: tlsInsecure}
        // }
        //
        // tr.disable_keep_alive = disable_keep_alive
        //
        // client.Transport = tr
    }

    Ok(client.build()?)
}
