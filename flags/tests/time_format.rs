use flags::*;

#[test]
fn test_time_format() {
    struct CaseInfo {
        _name: &'static str,
        value: &'static str,
        expected: &'static str,
    }
    let cases = vec![
        CaseInfo {
            _name: "can parse short name rfc850",
            value: "rfc850",
            expected: "Monday, 02-Jan-06 15:04:05 MST",
        },
        CaseInfo {
            _name: "can accept an arbitrary format string",
            value: "2006-01-02 15:04:05.999999999 -0700 MST",
            expected: "2006-01-02 15:04:05.999999999 -0700 MST",
        },
        CaseInfo {
            _name: "can accept arbitrary string",
            value: "nonsense",
            expected: "nonsense",
        },
    ];
    for case in cases {
        let mut format: Option<TimeFormat> = None;
        let res = format.set(case.value);
        if res.is_ok() {
            assert_eq!(format.string(), case.expected);
        } else {
            assert!(false);
        }
    }
}
