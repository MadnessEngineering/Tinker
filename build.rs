use std::process::Command;

fn main() {
    // Get git hash
    let git_hash = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string());

    // Pass git hash to the compiler
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // Tell cargo to rerun this script only if the .git/HEAD file changes
    println!("cargo:rerun-if-changed=.git/HEAD");
} 