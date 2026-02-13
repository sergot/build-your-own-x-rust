use std::process::Command as StdCommand;

use assert_cmd::Command;

pub fn assert_matches_git(args: &[&str]) {
    let git_output = StdCommand::new("git")
        .args(args)
        .output()
        .expect("git command should run successfully");

    assert!(
        git_output.status.success(),
        "git command failed with {:?}",
        args
    );

    let expected_output =
        String::from_utf8(git_output.stdout).expect("git output should be valid utf8");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("build-your-own-git"));
    cmd.args(args).assert().success().stdout(expected_output);
}
