use chrono::{DateTime, FixedOffset};
use flags::*;

#[test]
fn test_timestamp() {
    struct DateInfo {
        _name: &'static str,
        value: &'static str,
        expected: DateTime<FixedOffset>,
        _res: utility::Result<()>,
    }
    let cases = vec![
        DateInfo {
            _name: "valid rfc3339 parses",
            value: "2012-01-02T10:01:12Z",
            expected: DateTime::parse_from_rfc3339("2012-01-02T10:01:12Z").unwrap(),
            _res: Ok(()),
        },
        DateInfo {
            _name: "valid rfc3339 parses",
            value: "2012-01-02T10:01:12Z",
            expected: DateTime::parse_from_rfc3339("2012-01-02T10:01:12Z").unwrap(),
            _res: Ok(()),
        },
        DateInfo {
            _name: "in-valid rfc3339 parses",
            value: "2012-01-02T10:01:12Z",
            expected: DateTime::parse_from_rfc3339("2012-01-02T10:01:12Z").unwrap(),
            _res: Ok(()),
        },
    ];
    for case in cases {
        let mut timestamp: Option<TimestampFlag> = None;
        assert!(timestamp.set(case.value).is_ok());
        assert_eq!(
            timestamp.as_time().unwrap().to_string(),
            case.expected.to_string()
        );
    }
}
