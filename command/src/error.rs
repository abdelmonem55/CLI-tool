#![allow(dead_code)]
/// NoTLSWarn Warning thrown when no SSL/TLS is used
pub(crate) const NOT_TLS_WARN: &str =
    "WARNING! You are not using an encrypted connection to the gateway, consider using HTTPS.";

/// checktls_insecure returns a warning message if the given gateway does not have https.
/// Use tsInsecure to skip validations
pub(crate) fn check_tls_insecure(gateway: &str, tls_insecure: bool) -> String {
    let mut res = String::new();
    if !tls_insecure {
        if !gateway.starts_with("https")
            && !gateway.starts_with("http://127.0.0.1")
            && !gateway.starts_with("http://localhost")
        {
            res = NOT_TLS_WARN.to_string();
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use crate::error::check_tls_insecure;

    #[test]
    fn test_check_tls_insecure() {
        struct Args {
            gateway: &'static str,
            tls_insecure: bool,
        }
        struct TestCase {
            _name: &'static str,
            args: Args,
            want: &'static str,
        }
        let tests = vec![
            TestCase{
                _name: "HTTPS gateway",
                args: Args{gateway: "https://192.168.0.101:8080", tls_insecure: false},
                want: ""},
            TestCase{
                _name: "HTTPS gateway with tls_insecure",
                args: Args{gateway: "https://192.168.0.101:8080", tls_insecure: true},
                want: ""},
            TestCase{
                _name: "HTTP gateway without tls_insecure",
                args: Args{gateway: "http://192.168.0.101:8080", tls_insecure: false},
                want: "WARNING! You are not using an encrypted connection to the gateway, consider using HTTPS."},
            TestCase{
                _name: "HTTP gateway to 127.0.0.1 without tls_insecure",
                args: Args{gateway: "http://127.0.0.1:8080", tls_insecure: false},
                want: ""},
            TestCase{
                _name: "HTTP gateway to localhost without tls_insecure",
                args: Args{gateway: "http://localhost:8080", tls_insecure: false},
                want: ""},
            TestCase{
                _name: "HTTP gateway to remote host with tls_insecure",
                args: Args{gateway: "http://192.168.0.101:8080", tls_insecure: true},
                want: ""}
        ];

        for case in tests {
            //println!("{:?}",case._name);
            let got = check_tls_insecure(case.args.gateway, case.args.tls_insecure);
            assert_eq!(got, case.want);
        }
    }
}
