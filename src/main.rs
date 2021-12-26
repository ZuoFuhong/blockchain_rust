use blockchain_rust::{
    calc_address, hash_pub_key, utils, validate_address, Blockchain, Transaction, Wallets,
    ADDRESS_CHECK_SUM_LEN,
};
use data_encoding::HEXLOWER;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "blockchain_rust")]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "createblockchain", about = "Create a new blockchain")]
    Createblockchain,
    #[structopt(name = "createwallet", about = "Create a new wallet")]
    Createwallet,
    #[structopt(
        name = "getBalance",
        about = "Get the wallet balance of the target address"
    )]
    GetBalance {
        #[structopt(name = "address", help = "The wallet address")]
        address: String,
    },
    #[structopt(name = "listaddresses", about = "Print local wallet addres")]
    ListAddresses,
    #[structopt(name = "send", about = "Add new block to chain")]
    Send {
        #[structopt(name = "from", help = "Source wallet address")]
        from: String,
        #[structopt(name = "to", help = "Destination wallet address")]
        to: String,
        #[structopt(name = "amount", help = "Amount to send")]
        amount: i32,
    },
    #[structopt(name = "printchain", about = "Print blockchain all block")]
    Printchain,
    #[structopt(name = "clearchain", about = "Print blockchain all block")]
    Clearchain,
}

fn main() {
    let opt = Opt::from_args();
    match opt.command {
        Command::Createblockchain => {
            let _ = Blockchain::new_blockchain();
            println!("Done!");
        }
        Command::Createwallet => {
            let mut wallet = Wallets::new();
            let address = wallet.create_wallet();
            wallet.save_to_file();
            println!("Your new address: {}", address)
        }
        Command::GetBalance { address } => {
            let blockchain = Blockchain::new_blockchain();
            let payload = utils::base58_decode(address.as_str());
            let pub_key_hash = payload[1..payload.len() - ADDRESS_CHECK_SUM_LEN].to_vec();
            let utxos = blockchain.find_utxo(pub_key_hash.as_slice());
            let mut balance = 0;
            for utxo in utxos {
                balance += utxo.get_value();
            }
            println!("Balance of {}: {}", address, balance);
        }
        Command::ListAddresses => {
            let wallets = Wallets::new();
            for address in wallets.get_addresses() {
                println!("{}", address)
            }
        }
        Command::Send { from, to, amount } => {
            if !validate_address(from.as_str()) {
                panic!("ERROR: Sender address is not valid")
            }
            if !validate_address(to.as_str()) {
                panic!("ERROR: Recipient address is not valid")
            }
            let mut blockchain = Blockchain::new_blockchain();
            let transaction =
                Transaction::new_utxo_transaction(from.as_str(), to.as_str(), amount, &blockchain);
            blockchain.mine_block(vec![transaction]);
            println!("Success!")
        }
        Command::Printchain => {
            let mut block_iterator = Blockchain::new_blockchain().iterator();
            loop {
                let option = block_iterator.next();
                if option.is_none() {
                    break;
                }
                let block = option.unwrap();
                println!("Pre block hash: {}", block.get_pre_block_hash());
                println!("Cur block hash: {}", block.get_hash());
                for tx in block.get_transactions() {
                    for input in tx.get_vin() {
                        let txid_hex = HEXLOWER.encode(input.get_txid().as_slice());
                        let pub_key_hash = hash_pub_key(input.get_pub_key().as_slice());
                        let address = calc_address(pub_key_hash.as_slice());
                        println!(
                            "Transaction input txid = {}, vout = {}, from = {}",
                            txid_hex,
                            input.get_vout(),
                            address,
                        )
                    }
                    let cur_txid_hex = HEXLOWER.encode(tx.get_id().as_slice());
                    for output in tx.get_vout() {
                        let pub_key_hash = output.get_pub_key_hash();
                        let address = calc_address(pub_key_hash.as_slice());
                        println!(
                            "Transaction output current txid = {}, value = {}, to = {}",
                            cur_txid_hex,
                            output.get_value(),
                            address,
                        )
                    }
                }
                println!("Timestamp: {}\n", block.get_timestamp());
            }
        }
        Command::Clearchain => {
            let blockchain = Blockchain::new_blockchain();
            blockchain.clear_data();
            println!("Done!");
        }
    }
}
