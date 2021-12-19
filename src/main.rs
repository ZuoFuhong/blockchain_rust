use blockchain_rust::Blockchain;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "blockchain_rust")]
struct Opt {
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    #[structopt(name = "addblock", about = "Add new block to chain")]
    Addblock {
        #[structopt(name = "data", help = "The string value of the block data")]
        data: String,
    },
    #[structopt(name = "printchain", about = "Print blockchain all block")]
    Printchain,
}

fn main() {
    let opt = Opt::from_args();
    match opt.command {
        Command::Addblock { data } => {
            let mut blockchain = Blockchain::new_blockchain();
            blockchain.add_block(data);
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
                println!("Data: {}", block.get_data());
                println!("Timestamp: {}\n", block.get_timestamp());
            }
        }
    }
}
