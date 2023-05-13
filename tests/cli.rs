use assert_cmd::cargo::CommandCargoExt;
use assert_cmd::prelude::*;
use predicates::str::contains;
use rchain::wallet::{Wallet, Wallets};
use std::process::Command;
use tempfile::TempDir;

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

    let init_address;
    let address_1;
    let address_2;
    {
        let wallets = Wallets::with_path(temp_dir.path());

        let init_wallet = Wallet::new();
        init_address = init_wallet.address();
        wallets.set(&init_wallet).unwrap();

        let wallet1 = Wallet::new();
        address_1 = wallet1.address();
        wallets.set(&wallet1).unwrap();
        let wallet2 = Wallet::new();
        address_2 = wallet2.address();
        wallets.set(&wallet2).unwrap();
    }

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["create-blockchain", &init_address])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["send", &init_address, &address_1, "5"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["send", &init_address, &address_2, "4"])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["balance", &init_address])
        .current_dir(&temp_dir)
        .assert()
        .stdout(contains("1"));

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
        .stdout(contains("4"));
}

#[test]
fn cli_send_no_enough() {
    let temp_dir = TempDir::new().unwrap();
    let init_address;
    let address_1;
    {
        let wallets = Wallets::with_path(temp_dir.path());

        let init_wallet = Wallet::new();
        init_address = init_wallet.address();
        wallets.set(&init_wallet).unwrap();

        let wallet1 = Wallet::new();
        address_1 = wallet1.address();
        wallets.set(&wallet1).unwrap();
    }

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["create-blockchain", &init_address])
        .current_dir(&temp_dir)
        .assert()
        .success();

    Command::cargo_bin("rchain")
        .unwrap()
        .args(&["send", &init_address, &address_1, "15"])
        .current_dir(&temp_dir)
        .assert()
        .stderr(contains("NoEnoughBalance"));
}
