use chrono::TimeZone;
use command::logs_format::json_format_message;
use utility::faas_provider::logs::Message;

#[test]
fn test_json_log_formatter() {
    let now = chrono::offset::Local::now();
    //now := time.Now()

    let message = Message {
        timestamp: now.to_string().into(),
        name: "test-func".into(),
        instance: "123test".into(),
        text: "test message\n".into(),
        ..Default::default()
    };
    let msg = serde_json::to_string(&message).unwrap();
    struct TestCase {
        _name: &'static str,
        time_format: &'static str,
        include_name: bool,
        include_instance: bool,
        expected: String,
    }

    let cases = vec![
        TestCase {
            _name: "default behavior",
            time_format: "rfc3339",
            include_name: true,
            include_instance: true,
            expected: msg.clone(),
        },
        TestCase {
            _name: "default behavior with all empty options",
            time_format: "",
            include_name: false,
            include_instance: false,
            expected: msg.clone(),
        },
    ];
    for case in cases {
        println!("test case: {}", case._name);
        let formatted = json_format_message(
            &message,
            case.time_format,
            case.include_name,
            case.include_instance,
        );
        assert!(formatted.is_ok());
        let formatted = formatted.unwrap();
        assert_eq!(formatted, case.expected);
    }
}

// fn test_plain_log_formatter() {
//     use chrono::{Utc,TimeZone};
//     let ts =chrono::Local::now();
//     ts.ymd(1970, 1, 1).and_hms_milli(0, 0, 1, 444)
//     let ts = chrono::offset::FixedOffset::
//     ts := time.Date(2009, time.November, 10, 23, 0, 0, 0, time.UTC)
//     msg := logs.Message{
//         Timestamp: ts,
//         Name:      "test-func",
//         Instance:  "123test",
//         Text:      "test message",
//     }
//
//     cases := []struct {
//         name            string
//         timeFormat      string
//         includeName     bool
//         includeInstance bool
//         want            string
//     }{
//         {"default settings", time.RFC3339, true, true, "2009-11-10T23:00:00Z test-func (123test) test message"},
//         {"default can modify timestamp", "2006-01-02 15:04:05.999999999 -0700 MST", true, true, msg.String()},
//         {"can hide name", time.RFC3339, false, true, "2009-11-10T23:00:00Z (123test) test message"},
//         {"can hide instance", time.RFC3339, true, false, "2009-11-10T23:00:00Z test-func test message"},
//         {"can hide all metadata", "", false, false, "test message"},
//     }
//
//     for _, tc := range cases {
//         t.Run(tc.name, func(t *testing.T) {
//         got := PlainFormatMessage(msg, tc.timeFormat, tc.includeName, tc.includeInstance)
//         if strings.TrimSpace(got) != strings.TrimSpace(tc.want) {
//         t.Fatalf("incorrect message format:\n want %q\n got %q\n", tc.want, got)
//         }
//         })
//     }
// }

// fn Test_KeyValueLogFormatter(t *testing.T) {
//     ts := time.Date(2009, time.November, 10, 23, 0, 0, 0, time.UTC)
//     msg := logs.Message{
//         Timestamp: ts,
//         Name:      "test-func",
//         Instance:  "123test",
//         Text:      "test message\n",
//     }
//
//     cases := []struct {
//         name            string
//         timeFormat      string
//         includeName     bool
//         includeInstance bool
//         expected        string
//     }{
//         {"default settings", time.RFC3339, true, true, "timestamp=\"2009-11-10T23:00:00Z\" name=\"test-func\" instance=\"123test\" text=\"test message\""},
//         {"default settings", "2006-01-02 15:04:05.999999999 -0700 MST", true, true, "timestamp=\"2009-11-10 23:00:00 +0000 UTC\" name=\"test-func\" instance=\"123test\" text=\"test message\""},
//         {"can hide name", time.RFC3339, false, true, "timestamp=\"2009-11-10T23:00:00Z\" instance=\"123test\" text=\"test message\""},
//         {"can hide instance", time.RFC3339, true, false, "timestamp=\"2009-11-10T23:00:00Z\" name=\"test-func\" text=\"test message\""},
//         {"can hide all metadata", "", false, false, "text=\"test message\""},
//     }
//     for _, tc := range cases {
//         t.Run(tc.name, func(t *testing.T) {
//         formatted := KeyValueFormatMessage(msg, tc.timeFormat, tc.includeName, tc.includeInstance)
//         if strings.TrimSpace(formatted) != strings.TrimSpace(tc.expected) {
//         t.Fatalf("incorrect message format:\n got %s\n expected %s\n", formatted, tc.expected)
//         }
//         })
//     }
// }
