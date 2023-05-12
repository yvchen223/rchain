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
fn cli_balance() {
    let temp_dir = TempDir::new().unwrap();
    let genesis_address = temp_dir.path().file_name().unwrap().to_str().unwrap();
    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["balance", genesis_address])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains("10"));

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["balance", "who"])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains("0"));
}

#[test]
fn cli_send() {
    let temp_dir = TempDir::new().unwrap();
    let genesis_address = temp_dir.path().file_name().unwrap().to_str().unwrap();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["send", genesis_address, "b", "5"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["send", genesis_address, "c", "5"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["balance", genesis_address])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains("0"));

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["balance", "b"])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains("5"));

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["balance", "c"])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains("5"));
}

#[test]
fn cli_send_no_enough() {
    let temp_dir = TempDir::new().unwrap();
    let genesis_address = temp_dir.path().file_name().unwrap().to_str().unwrap();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["send", genesis_address, "b", "15"])
        .current_dir(&temp_dir)
        .assert()
        .stderr(contains("NoEnoughBalance"));
}

