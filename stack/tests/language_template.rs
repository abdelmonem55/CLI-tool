use stack::language_template::{is_valid_template, parse_yaml_data_for_language_template};
use stack::schema::LanguageTemplate;

#[test]
fn test_parse_yaml_data_for_language_template() {
    struct LangTemplateTest {
        input: &'static str,
        expected: LanguageTemplate,
    }
    let lang_templates = vec![
        LangTemplateTest {
            input: "
            language: python
            fprocess: python index.py
            ",
            expected: LanguageTemplate {
                language: "python".into(),
                fprocess: Some("python index.py".into()),
                build_options: vec![],
                welcome_message: "".to_string(),
                handler_folder: "".to_string(),
            },
        },
        LangTemplateTest {
            input: "
            language: python
            ",
            expected: LanguageTemplate {
                language: "python".into(),
                fprocess: None,
                build_options: vec![],
                welcome_message: "".to_string(),
                handler_folder: "".to_string(),
            },
        },
        LangTemplateTest {
            input: "
            fprocess: python index.py
            ",
            expected: LanguageTemplate {
                language: "".to_string(),
                fprocess: Some("python index.py".into()),
                build_options: vec![],
                welcome_message: "".to_string(),
                handler_folder: "".to_string(),
            },
        },
    ];
    for temp in lang_templates {
        let actual = parse_yaml_data_for_language_template(temp.input);
        //  println!("res {:?}",actual);
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual, temp.expected);
    }
}

#[tokio::test]
async fn test_is_valid_template() {
    assert!(!is_valid_template("unknown-temp").await);
    // let path = tempdir::TempDir::new("111111test_template").unwrap();
    //  let workdir = path.path().join("template/python");
    // println!("workdir {:?}",workdir.to_str().unwrap());
    std::fs::create_dir_all("./template/python").unwrap();
    let valid = is_valid_template("python").await;
    let _ = std::fs::remove_dir_all("./template/python");
    assert!(!valid);
}
