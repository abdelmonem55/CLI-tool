use proxy::utils::{create_function_endpoint, create_system_endpoint};

#[test]
fn test_create_system_endpoint() {
    struct TestCase {
        _title: &'static str,
        gateway: &'static str,
        namespace: &'static str,
        expected_err: bool,
        expected_endpoint: &'static str,
    }
    let cases = vec![
        TestCase {
            _title: "Namespace is set",
            gateway: "http://127.0.0.1:8080",
            namespace: "production",
            expected_err: false,
            expected_endpoint: "http://127.0.0.1:8080/system/functions?namespace=production",
        },
        TestCase {
            _title: "Namespace is not set",
            gateway: "http://127.0.0.1:8080",
            namespace: "",
            expected_err: false,
            expected_endpoint: "http://127.0.0.1:8080/system/functions",
        },
        TestCase {
            _title: "Bad gateway formatting",
            gateway: "127.0.0.1:8080",
            namespace: "production",
            expected_err: true,
            expected_endpoint: "",
        },
    ];

    for case in cases {
        let res = create_system_endpoint(case.gateway, case.namespace);
        match res {
            Ok(actual) => {
                assert_eq!(actual, case.expected_endpoint);
            }
            Err(_) => {
                assert!(case.expected_err);
            }
        }
    }
}

#[test]
fn test_create_function_endpoint() {
    struct Case {
        _title: &'static str,
        gateway: &'static str,
        namespace: &'static str,
        function_name: &'static str,
        expected_err: bool,
        expected_endpoint: &'static str,
    }
    let cases = vec![
        Case {
            _title: "Namespace is set",
            gateway: "http://127.0.0.1:8080",
            namespace: "production",
            function_name: "cows",
            expected_err: false,
            expected_endpoint: "http://127.0.0.1:8080/system/function/cows?namespace=production",
        },
        Case {
            _title: "Namespace is not set",
            gateway: "http://127.0.0.1:8080",
            function_name: "cows",
            namespace: "",
            expected_err: false,
            expected_endpoint: "http://127.0.0.1:8080/system/function/cows",
        },
        Case {
            _title: "Bad gateway formatting",
            gateway: "127.0.0.1:8080",
            namespace: "production",
            function_name: "",
            expected_err: true,
            expected_endpoint: "",
        },
    ];

    for case in cases {
        match create_function_endpoint(case.gateway, case.function_name, case.namespace) {
            Ok(endpoint) => {
                assert_eq!(endpoint, case.expected_endpoint);
            }
            Err(_) => {
                assert!(case.expected_err);
            }
        }
    }
}
