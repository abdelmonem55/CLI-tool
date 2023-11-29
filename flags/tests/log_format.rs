use flags::*;
use utility::{Error, Result};

#[test]
fn test_log_format() {
    struct CaseInfo {
        _name: &'static str,
        value: &'static str,
        error: Result<()>,
    }
    let cases = vec![
        CaseInfo {
            _name: "can accept plain",
            value: PLAIN_LOG_FORMAT,
            error: Ok(()),
        },
        CaseInfo {
            _name: "can accept keyvalue",
            value: KEY_VALUE_LOG_FORMAT,
            error: Ok(()),
        },
        CaseInfo {
            _name: "can accept json",
            value: JSON_LOG_FORMAT,
            error: Ok(()),
        },
        CaseInfo {
            _name: "unknown strings cause error string",
            value: "nonsense",
            error: Err(Error::Custom("unknown log format: nonsense".into())),
        },
    ];
    for case in cases {
        let mut format: Option<LogFormat> = None;
        let res = format.set(case.value);
        let test = if let Err(err) = res {
            if let Error::Custom(msg1) = err {
                if let Err(Error::Custom(msg2)) = case.error {
                    msg1 == msg2
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            format.string().as_str() == case.value
        };
        assert!(test);
    }
}
