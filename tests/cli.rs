use assert_cmd::cargo::CommandCargoExt;
use assert_cmd::prelude::*;
use predicates::str::contains;
use std::process::Command;
use tempfile::TempDir;
use rchain::wallet::Wallet;

const INIT_ADDRESS: &str = "1FbkP5rheSAtFonCjoNikSofyrGNHMUqzA";

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
    let address = INIT_ADDRESS.to_owned();
    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["balance", &address])
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

    let address_1 = Wallet::new().address();
    let address_2 = Wallet::new().address();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["send", INIT_ADDRESS, &address_1, "5"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["send", INIT_ADDRESS, &address_2, "5"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["balance", INIT_ADDRESS])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains("0"));

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["balance", &address_1])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains("5"));

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["balance", &address_2])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains("5"));
}

#[test]
fn cli_send_no_enough() {
    let temp_dir = TempDir::new().unwrap();
    let address = Wallet::new().address();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["send", INIT_ADDRESS, &address, "15"])
        .current_dir(&temp_dir)
        .assert()
        .stderr(contains("NoEnoughBalance"));
}

