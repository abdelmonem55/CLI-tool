use std::io;
use std::io::Write;
use utility::faas::types::model::FunctionStatus;

#[ignore]
#[test]
fn test_list() {
    let expected_list_response = vec![
        FunctionStatus {
            name: "func-test1".into(),
            image: "image-test-1".into(),
            replicas: 1,
            invocation_count: 3.,
            ..Default::default()
        },
        FunctionStatus {
            name: "func-test2".into(),
            image: "image-test-2".into(),
            replicas: 3,
            invocation_count: 999999.,
            ..Default::default()
        },
    ];
    //
    //
    //

    // let body = serde_json::to_string(&expected_list_response).unwrap();
    // let _mok = mockito::mock("GET", "system/functions")
    //     .with_status(200)
    //     .with_body(body)
    //     .create();
    // while  true {
    //
    // }
    //9ND6X9Tje2WAEuZ76R2IMR006 , admin

    let url = format!("http://127.0.0.1:1234");
    let output = std::process::Command::new(
        "C:/Users/AbdelmonemMohamed/CLionProjects/faas/target/debug/faas.exe",
    )
    .args(&["list", "--gateway", url.as_str()])
    .output()
    .unwrap();
    io::stdout().write_all(&output.stdout).unwrap();
    let str = std::str::from_utf8(&output.stdout).unwrap();
    println!("{}", str);
    // println!("out .........\n\n{}",str);
    assert!(str.contains(expected_list_response[0].name.as_str()))
    // io::stderr().write_all(&output.stderr).unwrap();
}

#[test]
fn test_list_errors() {
    let output = std::process::Command::new(
        "C:/Users/AbdelmonemMohamed/CLionProjects/faas/target/debug/testing.exe",
    )
    .args(&["list", "--gateway", "bad gateway"])
    .output()
    .unwrap();
    io::stdout().write_all(&output.stdout).unwrap();
    let str = std::str::from_utf8(&output.stdout).unwrap();
    assert!(str.contains("RelativeUrlWithoutBase"))
}
