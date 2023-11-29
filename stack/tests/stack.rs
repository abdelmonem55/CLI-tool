#[cfg(test)]
mod test {
    use lazy_static::lazy_static;
    use stack::stack::*;
    use std::sync::Arc;

    const TEST_DATA_1: &str = r#"version: 1.0
provider:
  name: openfaas
  gateway: http://127.0.0.1:8080
  network: "func_functions"

functions:
  url-ping:
    lang: python
    handler: ./sample/url-ping
    image: alexellis/faas-url-ping

  nodejs-echo:
    lang: node
    handler: ./sample/nodejs-echo
    image: alexellis/faas-nodejs-echo

  imagemagick:
    lang: dockerfile
    handler: ./sample/imagemagick
    image: functions/resizer
    fprocess: "convert - -resize 50% fd:1"

  ruby-echo:
    lang: ruby
    handler: ./sample/ruby-echo
    image: alexellis/ruby-echo

  abcd-eeee:
    lang: node
    handler: ./sample/abcd-eeee
    image: stuff2/stuff23423"
    "#;

    const TEST_DATA_2: &str = r#"version: 1.0
provider:
  name: openfaas
  gateway: http://127.0.0.1:8080
  network: "func_functions"
"#;

    const NO_MATCHES_ERROR_MSG: &str =
        "no functions matching --filter/--regex were found in the YAML file";
    const INVALID_REGEX_ERROR_MSG: &str = "regex parse error";

    #[derive(Debug)]
    struct ParseYamlTest {
        title: &'static str,
        search_term: &'static str,
        functions: &'static [&'static str],
        file: &'static str,
        expected_error: &'static str,
    }

    lazy_static! {
        static ref PARSE_YAML_TESTS: Arc<Vec<ParseYamlTest>> = std::sync::Arc::new(vec![
            ParseYamlTest {
                title: "Regex search for functions only containing 'node'",
                search_term: "node",
                functions: &["nodejs-echo"],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Regex search for functions only containing 'echo'",
                search_term: "echo",
                functions: &["nodejs-echo", "ruby-echo"],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Regex search for functions only containing '.+-.+'",
                search_term: ".+-.+",
                functions: &["abcd-eeee", "nodejs-echo", "ruby-echo", "url-ping"],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Regex search for all functions: '.*'",
                search_term: ".*",
                functions: &[
                    "abcd-eeee",
                    "imagemagick",
                    "nodejs-echo",
                    "ruby-echo",
                    "url-ping"
                ],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Regex search for no functions: '----'",
                search_term: "----",
                functions: &[],
                file: TEST_DATA_1,
                expected_error: NO_MATCHES_ERROR_MSG,
            },
            ParseYamlTest {
                title: "Regex search for functions without dashes: '^[^-]+$'",
                search_term: "^[^-]+$",
                functions: &["imagemagick"],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Regex search for functions with 8 characters: '^.{8}$'",
                search_term: "^.{8}$",
                functions: &["url-ping"],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Regex search for function with repeated 'e': 'e{2}'",
                search_term: "e{2}",
                functions: &["abcd-eeee"],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Regex empty search term: ''",
                search_term: "",
                functions: &[
                    "abcd-eeee",
                    "imagemagick",
                    "nodejs-echo",
                    "ruby-echo",
                    "url-ping"
                ],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Regex invalid regex 1: '['",
                search_term: "[",
                functions: &[],
                file: TEST_DATA_1,
                expected_error: INVALID_REGEX_ERROR_MSG,
            },
            ParseYamlTest {
                title: "Regex invalid regex 2: '*'",
                search_term: "*",
                functions: &[],
                file: TEST_DATA_1,
                expected_error: INVALID_REGEX_ERROR_MSG,
            },
            ParseYamlTest {
                title: "Regex invalid regex 3: '(\\w)\\1'",
                search_term: r#"(\w)\1"#,
                functions: &[],
                file: TEST_DATA_1,
                expected_error: INVALID_REGEX_ERROR_MSG,
            },
            ParseYamlTest {
                title: "Regex that finds no matches: 'RANDOMREGEX'",
                search_term: "RANDOMREGEX",
                functions: &[],
                file: TEST_DATA_1,
                expected_error: NO_MATCHES_ERROR_MSG,
            },
            ParseYamlTest {
                title: "Regex empty search term in empty YAML file: ",
                search_term: "",
                functions: &[],
                file: TEST_DATA_2,
                expected_error: ""
            }
        ]);
    }
    lazy_static! {
        static ref PARSE_YAML_FILTER_TESTS: Arc<Vec<ParseYamlTest>> = std::sync::Arc::new(vec![
            ParseYamlTest {
                title: "Wildcard search for functions ending with 'echo'",
                search_term: "*echo",
                functions: &["nodejs-echo", "ruby-echo"],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Wildcard search for functions with a - in between two words: '*-*'",
                search_term: "*-*",
                functions: &["abcd-eeee", "nodejs-echo", "ruby-echo", "url-ping"],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Wildcard search for specific function: 'imagemagick'",
                search_term: "imagemagick",
                functions: &["imagemagick"],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Wildcard search for all functions: '*'",
                search_term: "*",
                functions: &[
                    "abcd-eeee",
                    "imagemagick",
                    "nodejs-echo",
                    "ruby-echo",
                    "url-ping"
                ],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Wildcard empty search term: ''",
                search_term: "",
                functions: &[
                    "abcd-eeee",
                    "imagemagick",
                    "nodejs-echo",
                    "ruby-echo",
                    "url-ping"
                ],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Wildcard multiple wildcard characters: '**'",
                search_term: "**",
                functions: &[
                    "abcd-eeee",
                    "imagemagick",
                    "nodejs-echo",
                    "ruby-echo",
                    "url-ping"
                ],
                file: TEST_DATA_1,
                expected_error: "",
            },
            ParseYamlTest {
                title: "Wildcard that finds no matches: 'RANDOMTEXT'",
                search_term: "RANDOMTEXT",
                functions: &[],
                file: TEST_DATA_1,
                expected_error: NO_MATCHES_ERROR_MSG,
            },
            ParseYamlTest {
                title: "Wildcard empty search term in empty YAML file: ''",
                search_term: "",
                functions: &[],
                file: TEST_DATA_2,
                expected_error: "",
            },
        ]);
    }

    #[test]
    fn test_parse_yaml_data_regex() {
        //let test= PARSE_YAML_TESTS.get(10).unwrap();
        for test in PARSE_YAML_TESTS.iter() {
            let parsed_yaml_res = parse_yaml_data(test.file, test.search_term, "", true);
            // println!("{:?}",&parsed_yaml_res);
            match parsed_yaml_res {
                Ok(_parsed_yaml) => {
                    assert!(test.expected_error.is_empty());
                    // let mut keys:Vec<String>=parsed_yaml.functions.keys().map(|k| k.clone()).collect();
                    // let mut keys_str:Vec<String>=keys.clone();
                    //
                    // keys.sort_by(|s1,s2| s1.cmp(s2));
                    // assert_ne!(keys,keys_str);
                    // println!("{:?}",&parsed_yaml.functions);
                    // println!("keys {:?}",keys);
                }
                Err(e) => {
                    assert!(!test.expected_error.is_empty());
                    assert!(format!("{:?}", e).contains(test.expected_error));
                }
            }
        }
    }

    #[test]
    fn test_parse_yaml_data_filter() {
        //let test= PARSE_YAML_TESTS.get(10).unwrap();
        for test in PARSE_YAML_FILTER_TESTS.iter() {
            let parsed_yaml_res = parse_yaml_data(test.file, "", test.search_term, true);
            // println!("{:?}",&parsed_yaml_res);
            match parsed_yaml_res {
                Ok(_parsed_yaml) => {
                    assert!(test.expected_error.is_empty());
                    // let mut keys:Vec<String>=parsed_yaml.functions.keys().map(|k| k.clone()).collect();
                    // let mut keys_str:Vec<String>=keys.clone();
                    //
                    // keys.sort_by(|s1,s2| s1.cmp(s2));
                    // assert_ne!(keys,keys_str);
                    // println!("{:?}",&parsed_yaml.functions);
                    // println!("keys {:?}",keys);
                }
                Err(e) => {
                    assert!(!test.expected_error.is_empty());
                    assert!(format!("{:?}", e).contains(test.expected_error));
                }
            }
        }
    }

    #[test]
    fn test_parse_yaml_data_regex_and_filter() {
        let res = parse_yaml_data(TEST_DATA_1, ".*", "*", true);
        assert!(res.is_err())
    }

    #[test]
    fn test_parse_yaml_data_provider_values() {
        #[derive(Debug)]
        struct TestCase {
            title: &'static str,
            provider: &'static str,
            expected_error: &'static str,
            file: &'static str,
        }
        let test_cases = vec![
            TestCase {
                title: "Provider is openfaas and gives no error",
                provider: "openfaas",
                expected_error: "",
                file: r#"version: 1.0
provider:
  name: openfaas
  gateway: http://127.0.0.1:8080
  network: "func_functions"
"#,
            },
            TestCase {
                title: "Provider is openfaas and gives no error",
                provider: "openfaas",
                expected_error: "",
                file: r#"version: 1.0
provider:
  name: openfaas
  gateway: http://127.0.0.1:8080
  network: "func_functions"
"#,
            },
            TestCase {
                title: "Provider is faas and gives error",
                provider: "faas",
                expected_error: "[\'openfaas\'] is the only valid \'provider.name\' for the OpenFaaS CLI, but you gave: faas",
                file: r#"version: 1.0
provider:
  name: faas
  gateway: http://127.0.0.1:8080
  network: "func_functions"
"#
            },
            TestCase {
                title: "Provider is serverless and gives error",
                provider: "faas",
                expected_error: r#"['openfaas'] is the only valid 'provider.name' for the OpenFaaS CLI, but you gave: serverless"#,
                file: r#"version: 1.0
provider:
  name: serverless
  gateway: http://127.0.0.1:8080
  network: "func_functions"
"#,
            },
        ];
        for case in test_cases {
            let res = parse_yaml_data(case.file, ".*", "*", true);
            if !case.expected_error.is_empty() {
                if let Err(e) = res {
                    //println!("error {:?}\n{}",&e,&case.expected_error);
                    assert!(
                        format!("{:?}", e).contains(format!("{:?}", case.expected_error).as_str())
                    );
                }
            }
        }
    }

    #[test]
    fn test_substitute_environment_default_overridden() {
        std::env::set_var("USER", "alexellis2");

        let want = "alexellis2/image:latest";
        let template = "${USER:-openfaas}/image:latest";
        let res = substitute_vars(template);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), want);
        std::env::remove_var("USER");
    }

    #[test]
    fn test_substitute_environment_default_empty_overridden() {
        std::env::set_var("USER", "");
        let want = "openfaas/image:latest";
        let template = "${USER:-openfaas}/image:latest";
        let res = substitute_vars(template);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), want);
        std::env::remove_var("USER");
    }

    #[test]
    fn test_substitute_environment_default_unset_overridden() {
        std::env::remove_var("USER");
        let want = "openfaas/image:latest";
        let template = "${USER:-openfaas}/image:latest";
        let res = substitute_vars(template);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), want);
        std::env::remove_var("USER");
    }

    //  #[tokio::test]
    // async fn test_parse_yaml_file() {
    //     let res= parse_yaml_file("C:/Users/AbdelmonemMohamed/CLionProjects/faas/stack.yml","","",true).await;
    //      println!("{:?}",res);
    //
    //  }
}
