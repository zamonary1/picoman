use std::process::Command;
fn main() {
    // note: add error checking yourself.
    let output = Command::new("git").args(&["rev-parse", "HEAD"]).output();
    let git_hash: String;

    match output {
        Ok(output) => git_hash = String::from_utf8(output.stdout).unwrap(),
        Err(_) => git_hash = String::from("error_getting_git_hash"), // _ => (),
    }

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
