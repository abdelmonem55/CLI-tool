use config::config_file::*;
use std::collections::HashMap;

#[test]
fn test_lookup_auth_config_with_no_config_file() {
    let config_dir = tempdir::TempDir::new("faas-cli-file-test");
    assert!(config_dir.is_ok());

    let config_dir = config_dir.unwrap().path().to_string_lossy().into_owned();
    std::env::set_var(CONFIG_LOCATION_ENV, config_dir);

    let res = lookup_auth_config("http://openfaas.test1");
    assert!(res.is_err());
    let err = res.err().unwrap();
    //   println!("er {:?}",&err);
    assert!(format!("{:?}", err).contains("config file is not found"));
}
#[test]
#[ignore]
fn test_lookup_auth_config_gateway_with_no_config() {
    let config_dir = tempdir::TempDir::new("faas-cli-file-test");
    assert!(config_dir.is_ok());

    let config_dir = config_dir.unwrap().path().to_string_lossy().into_owned();
    std::env::set_var(CONFIG_LOCATION_ENV, config_dir);

    let u = "admin";
    let p = "some pass";
    let gateway = "http://openfaas.test/".trim_end_matches('/');

    let token = encode_auth(u, p);
    let res = update_auth_config(gateway, token.as_str(), BASIC_AUTH_TYPE.into());

    assert!(res.is_ok());

    let res = lookup_auth_config("http://openfaas.com");
    assert!(res.is_err());
    assert!(format!("{:?}", res)
        .contains(format!("no auth config found for {}", "http://openfaas.com").as_str()));
}

#[test]
#[ignore]
///ignored as it fails in running all test due to temp files creation
fn test_update_auth_config_update() {
    let res = tempdir::TempDir::new("faas-cli-file-test");
    assert!(res.is_ok());

    let config_dir = res.unwrap().path().to_string_lossy().into_owned();
    std::env::set_var(CONFIG_LOCATION_ENV, config_dir);

    let u = "admin";
    let p = "some pass";
    let gateway = "http://openfaas.test/".trim_end_matches('/');

    let token = encode_auth(u, p);
    let res = update_auth_config(gateway, token.as_str(), BASIC_AUTH_TYPE.into());

    assert!(res.is_ok());

    let res = lookup_auth_config(gateway);
    assert!(res.is_ok());
    let auth_config = res.unwrap();
    let res = decode_auth(auth_config.token.as_str());

    assert!(res.is_ok());
    let (user, pass) = res.unwrap();
    assert!(u == user && p == pass);
    let u = "admin2";
    let p = "pass2";
    let token = encode_auth(u, p);
    let res = update_auth_config(gateway, token.as_str(), BASIC_AUTH_TYPE.into());
    assert!(res.is_ok());
    let res = lookup_auth_config(gateway);
    assert!(res.is_ok());
    let auth_config = res.unwrap();

    let res = decode_auth(auth_config.token.as_str());

    assert!(res.is_ok());
    let (user, pass) = res.unwrap();
    assert!(u == user && p == pass);
}

#[test]
fn test_update_auth_config_invalid_gateway() {
    let gateway = "http//test.test";
    let res = update_auth_config(gateway, "a", "b".into());
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(format!("{:?}", err).contains("invalid gateway"));
}

#[test]
fn test_update_auth_config_empty_gateway() {
    let gateway = "";
    let res = update_auth_config(gateway, "a", "b".into());
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(format!("{:?}", err).contains("invalid gateway"));
}

#[test]
fn test_new_no_file() {
    let res = ConfigFile::new("".into());
    assert!(res.is_err())
}

#[test]
fn test_ensure_file() {
    let config_dir = tempdir::TempDir::new("faas-cli-file-test");
    assert!(config_dir.is_ok());

    let config_dir = config_dir.unwrap().path().to_string_lossy().into_owned();
    std::env::set_var(CONFIG_LOCATION_ENV, config_dir);

    let res = ensure_file();
    assert!(res.is_ok());
    let cfg = res.unwrap();

    let res = std::fs::metadata(cfg);
    if let Err(err) = res {
        assert!(!format!("{:?}", err).contains("kind: NotFound"));
    }
}
#[test]
fn test_encode_auth() {
    let token = encode_auth("admin", "admin");
    assert_eq!(token, "YWRtaW46YWRtaW4=");
}
#[test]
pub fn test_decode_auth() {
    let res = decode_auth("YWRtaW46YWRtaW4=");
    assert!(res.is_ok());
    assert_eq!(res.unwrap(), ("admin".into(), "admin".into()));
}

#[test]
#[ignore]
///ignored as it fails in running all test due to temp files creation
fn test_remove_auth_config() {
    let config_dir = tempdir::TempDir::new("faas-cli-file-test");
    assert!(config_dir.is_ok());

    let config_dir = config_dir.unwrap().path().to_string_lossy().into_owned();
    std::env::set_var(CONFIG_LOCATION_ENV, config_dir);

    let u = "admin";
    let p = "pass";
    let token = encode_auth(u, p);
    let gateway1 = "http://openfaas.test/".trim_end_matches('/');
    let res = update_auth_config(gateway1, token.as_str(), BASIC_AUTH_TYPE.into());
    assert!(res.is_ok());

    let gateway2 = "http://openfaas.test2/".trim_end_matches('/');
    let res = update_auth_config(gateway2, token.as_str(), BASIC_AUTH_TYPE.into());
    assert!(res.is_ok());
    let res = remove_auth_config(gateway1);
    assert!(res.is_ok());

    let res = lookup_auth_config(gateway1);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(format!("{:?}", err).contains("no auth config found"));
}

#[test]
fn test_remove_config_with_no_config_file() {
    let config_dir = tempdir::TempDir::new("faas-cli-file-test");
    assert!(config_dir.is_ok());

    let config_dir = config_dir.unwrap().path().to_string_lossy().into_owned();
    std::env::set_var(CONFIG_LOCATION_ENV, config_dir);

    let gateway = "http://openfaas.test2/".trim_end_matches('/');
    let res = remove_auth_config(gateway);
    assert!(res.is_err());

    let err = res.err().unwrap();
    assert!(format!("{:?}", err).contains("config file is not found"));
}
#[test]
#[ignore]
///ignored as it fails in running all test due to temp files creation
fn test_remove_config_with_unknown_gateway() {
    let config_dir = tempdir::TempDir::new("faas-cli-file-test");
    assert!(config_dir.is_ok());

    let config_dir = config_dir.unwrap().path().to_string_lossy().into_owned();
    std::env::set_var(CONFIG_LOCATION_ENV, config_dir);

    let u = "admin";
    let p = "pass";
    let token = encode_auth(u, p);
    let gateway1 = "http://openfaas.test/".trim_end_matches('/');
    let res = update_auth_config(gateway1, token.as_str(), BASIC_AUTH_TYPE.into());
    assert!(res.is_ok());

    let gateway2 = "http://openfaas.test2/".trim_end_matches('/');
    let res = remove_auth_config(gateway2);
    assert!(res.is_err());
    let err = res.err().unwrap();
    assert!(format!("{:?}", err).contains("not found in config"));
}

#[test]
#[ignore]
///ignored as it fails in running all test due to temp files creation
fn test_update_config_oath2_insert() {
    let config_dir = tempdir::TempDir::new("faas-cli-file-test");
    assert!(config_dir.is_ok());

    let config_dir = config_dir.unwrap().path().to_string_lossy().into_owned();
    std::env::set_var(CONFIG_LOCATION_ENV, config_dir);

    let token = "somebase64encodedstring";
    let gateway = "http://openfaas.test/".trim_end_matches('/');
    let res = update_auth_config(gateway, token, BASIC_AUTH_TYPE.into());
    assert!(res.is_ok());

    let res = lookup_auth_config(gateway);
    assert!(res.is_ok());
    assert_eq!(token, res.unwrap().token);
}

#[test]
fn test_config_dir() {
    struct TestCase {
        _name: &'static str,
        env: HashMap<&'static str, &'static str>,
        expected_path: &'static str,
    }
    let cases = vec![
        TestCase {
            _name: "override value is returned",
            env: [("OPENFAAS_CONFIG", "/tmp/foo")].iter().cloned().collect(),
            expected_path: "/tmp/foo",
        },
        TestCase {
            _name: "override value is returned, when CI is set but false",
            env: [("OPENFAAS_CONFIG", "/tmp/foo"), ("CI", "false")]
                .iter()
                .cloned()
                .collect(),
            expected_path: "/tmp/foo",
        },
        TestCase {
            _name: "override value is returned even when CI is set",
            env: [("OPENFAAS_CONFIG", "/tmp/foo"), ("CI", "true")]
                .iter()
                .cloned()
                .collect(),
            expected_path: "/tmp/foo",
        },
        TestCase {
            _name: "when CI is true, return the default CI directory",
            env: [("CI", "true")].iter().cloned().collect(),
            expected_path: DEFAULT_CI_DIR,
        },
        TestCase {
            _name: "when CI is false, return the default directory",
            env: [("CI", "false")].iter().cloned().collect(),
            expected_path: DEFAULT_DIR,
        },
        TestCase {
            _name: "when no other env variables are set, the default path is returned",
            env: HashMap::new(),
            expected_path: DEFAULT_DIR,
        },
    ];

    for case in cases {
        for (name, value) in &case.env {
            std::env::set_var(name, value);
        }

        let path = config_dir();
        assert!(path.is_ok());
        let path = path.unwrap();
        assert_eq!(path, case.expected_path);
        for (name, _) in case.env {
            std::env::remove_var(name);
        }
    }
}
