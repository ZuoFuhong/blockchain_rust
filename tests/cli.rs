use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use std::process::Command;

#[test]
fn client_createblockchain() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("createblockchain")
        .arg("1HvuWmqQrmjFGD8joWhv9ZfU1BYipM2iWZ")
        .assert()
        .success();
}

#[test]
fn client_createwallet() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("createwallet")
        .assert()
        .success();
}

#[test]
fn client_get_addresses() {
    let assert = Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("listaddresses")
        .assert()
        .success();

    let output_bytes = assert.get_output().stdout.as_slice();
    println!("{}", String::from_utf8(output_bytes.to_vec()).unwrap())
}

#[test]
fn client_get_balance() {
    let command = Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("getbalance")
        .arg("1HvuWmqQrmjFGD8joWhv9ZfU1BYipM2iWZ")
        .assert()
        .success();

    let output_bytes = command.get_output().stdout.as_slice();
    println!("{}", String::from_utf8(output_bytes.to_vec()).unwrap())
}

#[test]
fn client_send() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("send")
        .arg("1HvuWmqQrmjFGD8joWhv9ZfU1BYipM2iWZ")
        .arg("1NsavE9u2qSTzJCmMMZVKHfGsAAAE36XYo")
        .arg("5")
        .arg("0")
        .assert()
        .success();
}

#[test]
fn client_printchain() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("printchain")
        .assert()
        .success();
}

#[test]
fn client_reindexutxo() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("reindexutxo")
        .assert()
        .success();
}
