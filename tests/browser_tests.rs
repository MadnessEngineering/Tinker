use assert_cmd::Command;

#[test]
fn test_browser_headless_startup() {
    let mut cmd = Command::cargo_bin("tinker").unwrap();
    cmd.arg("--headless")
        .assert()
        .success();
} 
