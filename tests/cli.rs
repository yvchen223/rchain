use assert_cmd::cargo::CommandCargoExt;
use assert_cmd::prelude::*;
use predicates::str::contains;
use std::process::Command;
use tempfile::TempDir;

#[test]
fn cli_ls_chain() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["ls"])
        .current_dir(&temp_dir)
        .assert()
        .success();
}

#[test]
fn cli_add() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["add", "add_data"])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains("add_data"));
}

#[test]
fn cli_invalid_subcommand() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["unknown"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn cli_version() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("rchain").unwrap();
    cmd.args(&["-V"])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn cli_no_args() {
    let temp_dir = TempDir::new().unwrap();
    let mut cmd = Command::cargo_bin("rchain").unwrap();
    cmd.current_dir(&temp_dir).assert().failure();
}

#[test]
fn cli_invalid_ls() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["ls", "arg"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}

#[test]
fn cli_invalid_add() {
    let temp_dir = TempDir::new().unwrap();
    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["add"])
        .current_dir(&temp_dir)
        .assert()
        .failure();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["add", "data1", "data2"])
        .current_dir(&temp_dir)
        .assert()
        .failure();
}
