use crate::transaction::TXOutput;
use crate::{Block, Transaction};
use data_encoding::HEXLOWER;
use sled::Db;
use std::collections::HashMap;
use std::env::current_dir;

const TIP_BLOCK_HASH_KEY: &str = "tip_block_hash";
const GENESIS_ADDRESS: &str = "Genesis";
const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

pub struct Blockchain {
    tip: String,
    db: Db,
}

impl Blockchain {
    /// 创建区块链
    pub fn new_blockchain() -> Blockchain {
        let db = sled::open(current_dir().unwrap().join("data")).unwrap();
        let data = db.get(TIP_BLOCK_HASH_KEY).unwrap();
        let tip;
        if data.is_none() {
            let coinbase_tx = Transaction::new_coinbase_tx(
                String::from(GENESIS_ADDRESS),
                String::from(GENESIS_COINBASE_DATA),
            );
            let block = Block::generate_genesis_block(coinbase_tx);
            let block_hash = block.get_hash();
            let _ = db.insert(block_hash.clone(), block);
            let _ = db.insert(TIP_BLOCK_HASH_KEY, block_hash.as_bytes().to_vec());
            tip = block_hash;
        } else {
            tip = String::from_utf8(data.unwrap().to_vec()).unwrap();
        }
        Blockchain { tip, db }
    }

    /// 挖矿新区块
    pub fn mine_block(&mut self, transactions: Vec<Transaction>) {
        let block = Block::new_block(self.tip.clone(), transactions);
        let block_hash = block.get_hash();
        let _ = self.db.insert(block_hash.clone(), block);
        let _ = self
            .db
            .insert(TIP_BLOCK_HASH_KEY, block_hash.as_bytes().to_vec());
        self.tip = block_hash;
    }

    pub fn iterator(&self) -> BlockchainIterator {
        BlockchainIterator::new(self.tip.clone(), self.db.clone())
    }

    /// 找到未花费支出的交易
    /// 1.有一些输出并没有被关联到某个输入上，如 coinbase 挖矿奖励。
    /// 2.一笔交易的输入可以引用之前多笔交易的输出。
    /// 3.一个输入必须引用一个输出。
    pub fn find_unspent_transactions(&self, address: &str) -> Vec<Transaction> {
        let mut unspent_txs = vec![];
        let mut spent_txos: HashMap<String, Vec<i32>> = HashMap::new();

        let mut block_iterator = self.iterator();
        loop {
            // 区块是从尾部向上
            let block = block_iterator.next();
            if block.is_none() {
                break;
            }
            for tx in block.unwrap().get_transactions() {
                // 未花费输出
                let txid_hex = HEXLOWER.encode(tx.get_id().as_slice());
                let txout = tx.get_vout();
                'outer: for idx in 0..txout.len() {
                    let txout = txout[idx].clone();

                    // 过滤已花费输出
                    if spent_txos.contains_key(txid_hex.as_str()) {
                        let outs = spent_txos.get(txid_hex.as_str()).unwrap();
                        for out in outs {
                            if out.eq(&(idx as i32)) {
                                continue 'outer;
                            }
                        }
                    }
                    if txout.can_be_unlocked_with(address) {
                        unspent_txs.push(tx.clone())
                    }
                }
                if tx.is_coinbase() {
                    continue;
                }
                // 在输入中查找已花费输出
                for txin in tx.get_vin() {
                    if txin.can_unlock_output_with(address) {
                        let txid_hex = HEXLOWER.encode(txin.get_txid().as_slice());
                        if spent_txos.contains_key(txid_hex.as_str()) {
                            let outs = spent_txos.get_mut(txid_hex.as_str()).unwrap();
                            outs.push(txin.get_vout());
                        } else {
                            spent_txos.insert(txid_hex, vec![txin.get_vout()]);
                        }
                    }
                }
            }
        }
        return unspent_txs;
    }

    pub fn find_utxo(&self, address: &str) -> Vec<TXOutput> {
        let transactions = self.find_unspent_transactions(address);
        let mut utxos = vec![];
        for transaction in transactions {
            for out in transaction.get_vout() {
                if out.can_be_unlocked_with(address) {
                    utxos.push(out);
                }
            }
        }
        return utxos;
    }

    pub fn clear_data(&self) {
        let _ = self.db.clear();
    }
}

pub struct BlockchainIterator {
    db: Db,
    current_hash: String,
}

impl BlockchainIterator {
    fn new(tip: String, db: Db) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: tip,
            db,
        }
    }

    pub fn next(&mut self) -> Option<Block> {
        let data = self.db.get(self.current_hash.clone()).unwrap();
        if data.is_none() {
            return None;
        }
        let block = Block::deserialize(data.unwrap().to_vec().as_slice());
        self.current_hash = block.get_pre_block_hash().clone();
        return Some(block);
    }
}

#[cfg(test)]
mod tests {
    use crate::Transaction;
    use data_encoding::HEXLOWER;
    use std::env::current_dir;

    #[test]
    fn test_blockchain() {
        let mut blockchain = super::Blockchain::new_blockchain();
        // 创建一个 coinbase 交易
        let transaction = Transaction::new_coinbase_tx(String::from("mars"), String::from("miko"));
        blockchain.mine_block(vec![transaction]);
    }

    #[test]
    fn test_find_unspent_transactions() {
        let blockchain = super::Blockchain::new_blockchain();
        let transactions = blockchain.find_unspent_transactions("mars");
        for transaction in transactions {
            let txid_hex = HEXLOWER.encode(transaction.get_id().as_slice());
            println!("txid = {}", txid_hex)
        }
    }

    #[test]
    fn test_sled() {
        let db = sled::open(current_dir().unwrap().join("data")).unwrap();
        let ret = db.get("name").unwrap();
        if ret.is_none() {
            println!("Not found")
        }
        let _ = db.insert("name", "mars");
        if let Some(v) = db.get("name").unwrap() {
            println!("data = {}", String::from_utf8(v.to_vec()).unwrap());
            let _ = db.remove("name");
        }
    }
}
