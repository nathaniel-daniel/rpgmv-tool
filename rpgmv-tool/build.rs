use std::path::PathBuf;
use std::process::Command;

fn find_git_repo() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("failed to spawn git");

    let mut stdout = String::from_utf8(output.stdout).expect("stdout is not utf8");
    if stdout.ends_with("\n") {
        stdout.pop();
    }
    if stdout.ends_with("\r") {
        stdout.pop();
    }
    let stderr = String::from_utf8(output.stderr).expect("stderr is not utf8");

    if !output.status.success() {
        panic!("invalid status code {0}: {stderr}", output.status);
    }

    stdout
}

fn get_git_rev() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("failed to spawn git");
    let mut stdout = String::from_utf8(output.stdout).expect("stdout is not utf8");
    if stdout.ends_with("\n") {
        stdout.pop();
    }
    if stdout.ends_with("\r") {
        stdout.pop();
    }
    let stderr = String::from_utf8(output.stderr).expect("stderr is not utf8");

    if !output.status.success() {
        panic!("invalid status code {0}: {stderr}", output.status);
    }

    stdout
}

fn main() {
    let git_base_dir = PathBuf::from(find_git_repo());

    let git_head = git_base_dir.join(".git").join("HEAD");
    println!(
        "cargo::rerun-if-changed={}",
        git_head.to_str().expect("path is not unicode")
    );
    let head_content = std::fs::read_to_string(git_head).expect("failed to read git head file");
    if let Some(git_ref) = head_content
        .strip_prefix("ref: ")
        .map(|git_ref| git_ref.trim())
    {
        let git_ref_path = git_base_dir.join(".git").join(git_ref);
        println!(
            "cargo::rerun-if-changed={}",
            git_ref_path.to_str().expect("path is not unicode")
        );
    }

    println!("cargo::rustc-env=GIT_REV={}", get_git_rev());
}
