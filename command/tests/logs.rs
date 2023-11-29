use utility::faas_provider::logs::Request;

#[ignore]
#[test]
fn test_logs_cmd_flag_parsing() {
    // nowFunc = func() time.Time {
    //     ts, _ := time.Parse(time.RFC3339, "2019-01-01T01:00:00Z")
    //     return ts
    // }
    let _ts = chrono::DateTime::parse_from_rfc3339("2019-01-01T01:00:00Z").unwrap();

    let five_min_ago_str = "2019-01-01T00:55:00Z";
    let five_min_ago = chrono::DateTime::parse_from_rfc3339(five_min_ago_str).unwrap();

    struct TestCase<'s> {
        name: &'s str,
        args: Vec<&'s str>,
        expected: Request<'s>,
    }
    let t3 = "--since-time=".to_string() + five_min_ago_str;
    let scenarios = vec![
        TestCase {
            name: "name only passed, follow on by default",
            args: vec!["funcFoo"],
            expected: Request {
                name: "funcFoo",
                follow: true,
                tail: -1,
                ..Default::default()
            },
        },
        TestCase {
            name: "can disable follow",
            args: vec!["funcFoo", "--tail=false"],
            expected: Request {
                name: "funcFoo",
                follow: false,
                tail: -1,
                ..Default::default()
            },
        },
        TestCase {
            name: "can limit number of messages returned",
            args: vec!["funcFoo", "--lines=5"],
            expected: Request {
                name: "funcFoo",
                follow: true,
                tail: 5,
                ..Default::default()
            },
        },
        TestCase {
            name: "can set timestamp to send logs since using duration",
            args: vec!["funcFoo", "--since=5m"],
            expected: Request {
                name: "funcFoo",
                follow: true,
                tail: -1,
                since: Some(five_min_ago.timestamp()),
                ..Default::default()
            },
        },
        TestCase {
            name: "can set timestamp to send logs since using timestamp",
            args: vec!["funcFoo", t3.as_str()],
            expected: Request {
                name: "funcFoo",
                follow: true,
                tail: -1,
                since: Some(five_min_ago.timestamp()),
                ..Default::default()
            },
        },
    ];

    for case in scenarios {
        println!("test case: {}", case.name);
        let url = format!("http://127.0.0.1:1234");
        let output = std::process::Command::new(
            "C:/Users/AbdelmonemMohamed/CLionProjects/faas/target/debug/testing.exe",
        )
        .args(&["logs", "--name"])
        .args(&case.args)
        .args(&["--gateway", url.as_str()])
        .output()
        .unwrap();
        //  std::io::stdout().write_all(&output.stdout).unwrap();
        let str = std::str::from_utf8(&output.stdout).unwrap();
        //println!("output {}",str);
        // println!("out .........\n\n{}",str);
        assert!(str.contains(case.expected.name))
    }

    // for _, s := range scenarios {
    //     t.Run(s.name, func(t *testing.T) {
    //     functionLogsCmd.ResetFlags()
    //
    //     initLogCmdFlags(functionLogsCmd)
    //     functionLogsCmd.ParseFlags(s.args)
    //
    //     logRequest := logRequestFromFlags(functionLogsCmd, functionLogsCmd.Flags().Args())
    //     if logRequest.String() != s.expected.String() {
    //     t.Errorf("expected log request %s, got %s", s.expected, logRequest)
    //     }
    //     })
    // }
}
