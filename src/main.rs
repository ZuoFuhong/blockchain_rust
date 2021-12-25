use blockchain_rust::{Blockchain, Transaction};
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
    #[structopt(
        name = "getBalance",
        about = "Get the wallet balance of the target address"
    )]
    GetBalance {
        #[structopt(name = "address", help = "The wallet address")]
        address: String,
    },
    #[structopt(name = "send", about = "Add new block to chain")]
    Send {
        #[structopt(name = "from", help = "The string value of the block data")]
        from: String,
        #[structopt(name = "to", help = "The string value of the block data")]
        to: String,
        #[structopt(name = "data", help = "The string value of the block data")]
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
        Command::GetBalance { address } => {
            let blockchain = Blockchain::new_blockchain();
            let utxos = blockchain.find_utxo(address.as_str());
            let mut balance = 0;
            for utxo in utxos {
                balance += utxo.get_value();
            }
            println!("Balance of {}: {}", address, balance);
        }
        Command::Send { from, to, amount } => {
            let mut blockchain = Blockchain::new_blockchain();
            let unspent_transactions = blockchain.find_unspent_transactions(from.as_str());
            let transaction =
                Transaction::new_utxo_transaction(from, to, amount, unspent_transactions);
            blockchain.mine_block(vec![transaction]);
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
                        println!(
                            "Transaction input txid = {}, vout = {}, script_sig = {}",
                            txid_hex,
                            input.get_vout(),
                            input.get_script_sig()
                        )
                    }
                    let cur_txid_hex = HEXLOWER.encode(tx.get_id().as_slice());
                    for output in tx.get_vout() {
                        println!(
                            "Transaction output current txid = {}, value = {}, script_pub_key = {}",
                            cur_txid_hex,
                            output.get_value(),
                            output.get_script_pub_key()
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
