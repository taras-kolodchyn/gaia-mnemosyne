use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Git commit hash
    if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if output.status.success() {
            if let Ok(hash) = String::from_utf8(output.stdout) {
                println!("cargo:rustc-env=MNEMO_GIT_COMMIT={}", hash.trim());
            }
        }
    }

    // Build timestamp (UTC seconds since epoch)
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    println!("cargo:rustc-env=MNEMO_BUILD_TIME={}", ts);
}
