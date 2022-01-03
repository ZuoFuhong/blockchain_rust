use assert_cmd::prelude::{CommandCargoExt, OutputAssertExt};
use std::process::Command;

#[test]
fn client_createblockchain() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("createblockchain")
        .arg("1NA3ZoWS1xHkvhTPXU4PEX9ABR5gJ1CHMR")
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
        .arg("1NA3ZoWS1xHkvhTPXU4PEX9ABR5gJ1CHMR")
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
        .arg("1NA3ZoWS1xHkvhTPXU4PEX9ABR5gJ1CHMR")
        .arg("1PipUzybS5DcMvNQe3XMiLML9z7fZdpz35")
        .arg("5")
        .arg("1")
        .assert()
        .success();
}

#[test]
fn client_printchain() {
    let command = Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("printchain")
        .assert()
        .success();

    let output_bytes = command.get_output().stdout.as_slice();
    println!("{}", String::from_utf8(output_bytes.to_vec()).unwrap())
}

#[test]
fn client_reindexutxo() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("reindexutxo")
        .assert()
        .success();
}

#[test]
fn client_startnode() {
    Command::cargo_bin(env!("CARGO_PKG_NAME"))
        .unwrap()
        .arg("startnode")
        .arg("1NA3ZoWS1xHkvhTPXU4PEX9ABR5gJ1CHMR")
        .assert()
        .success();
}
