use blockchain_rust::{Blockchain, UTXOSet, Wallets};
use ring::io::der::Tag::UTCTime;
use std::env::current_dir;
use std::process::Command;

#[test]
fn client_createblockchain() {
    let blockchain = Blockchain::create_blockchain();
    let _ = UTXOSet::new(blockchain);
}

#[test]
fn client_createwallet() {
    let mut wallets = Wallets::new();
    let address = wallets.create_wallet();
    println!("{}", address)
}

#[test]
fn client_get_addresses() {
    let wallets = Wallets::new();
    let addresses = wallets.get_addresses();
    println!("{:?}", addresses)
}
