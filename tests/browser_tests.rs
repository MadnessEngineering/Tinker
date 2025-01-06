use assert_cmd::Command;
use std::time::Duration;

#[test]
fn test_browser_headless_startup() {
    let mut cmd = Command::cargo_bin("tinker").unwrap();
    cmd.arg("--headless")
        .timeout(Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_browser_with_url() {
    let mut cmd = Command::cargo_bin("tinker").unwrap();
    cmd.args(["--headless", "--url", "https://example.com"])
        .timeout(Duration::from_secs(5))
        .assert()
        .success();
}

#[test]
fn test_browser_with_multiple_tabs() {
    let mut cmd = Command::cargo_bin("tinker").unwrap();
    cmd.args(["--headless", "--tabs", "3"])
        .timeout(Duration::from_secs(5))
        .assert()
        .success();
}
