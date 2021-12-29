use crate::transaction::TXOutput;
use crate::{Block, Transaction, Wallets};
use data_encoding::HEXLOWER;
use sled::transaction::TransactionResult;
use sled::{Db, Tree};
use std::collections::HashMap;
use std::env::current_dir;

const TIP_BLOCK_HASH_KEY: &str = "tip_block_hash";
const BLOCKS_TREE: &str = "blocks";

#[derive(Clone)]
pub struct Blockchain {
    tip: String,
    db: Db,
}

impl Blockchain {
    /// 创建新的区块链
    pub fn create_blockchain() -> Blockchain {
        let db = sled::open(current_dir().unwrap().join("data")).unwrap();
        let blocks_tree = db.open_tree(BLOCKS_TREE).unwrap();

        let data = blocks_tree.get(TIP_BLOCK_HASH_KEY).unwrap();
        let tip;
        if data.is_none() {
            // 创世块的钱包
            let mut wallets = Wallets::new();
            let genesis_address = wallets.create_wallet();

            let coinbase_tx = Transaction::new_coinbase_tx(genesis_address.as_str());
            let block = Block::generate_genesis_block(coinbase_tx);
            Self::update_blocks_tree(&blocks_tree, &block);
            tip = block.get_hash();
        } else {
            tip = String::from_utf8(data.unwrap().to_vec()).unwrap();
        }
        Blockchain { tip, db }
    }

    fn update_blocks_tree(blocks_tree: &Tree, block: &Block) {
        let block_hash = block.get_hash();
        let _: TransactionResult<(), ()> = blocks_tree.transaction(|tx_db| {
            let _ = tx_db.insert(block_hash.as_bytes(), block.clone());
            let _ = tx_db.insert(TIP_BLOCK_HASH_KEY, block_hash.as_bytes());
            Ok(())
        });
    }

    /// 创建区块链实例
    pub fn new_blockchain() -> Blockchain {
        let db = sled::open(current_dir().unwrap().join("data")).unwrap();
        let blocks_tree = db.open_tree(BLOCKS_TREE).unwrap();
        let tip_bytes = blocks_tree
            .get(TIP_BLOCK_HASH_KEY)
            .unwrap()
            .expect("No existing blockchain found. Create one first.");
        let tip = String::from_utf8(tip_bytes.to_vec()).unwrap();
        Blockchain { tip, db }
    }

    pub fn get_db(&self) -> &Db {
        &self.db
    }

    /// 挖矿新区块
    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Block {
        for trasaction in &transactions {
            if trasaction.verify(self) == false {
                panic!("ERROR: Invalid transaction")
            }
        }
        let block = Block::new_block(self.tip.as_str(), transactions);
        let block_hash = block.get_hash();

        let blocks_tree = self.db.open_tree(BLOCKS_TREE).unwrap();
        Self::update_blocks_tree(&blocks_tree, &block);
        self.tip = block_hash;
        block
    }

    pub fn iterator(&self) -> BlockchainIterator {
        BlockchainIterator::new(self.tip.clone(), self.db.clone())
    }

    /// 查找所有未花费的交易输出 ( K -> txid_hex, V -> Vec<TXOutput )
    pub fn find_utxo(&self) -> HashMap<String, Vec<TXOutput>> {
        let mut utxo: HashMap<String, Vec<TXOutput>> = HashMap::new();
        let mut spent_txos: HashMap<String, Vec<usize>> = HashMap::new();

        let mut iterator = self.iterator();
        loop {
            let option = iterator.next();
            if option.is_none() {
                break;
            }
            let block = option.unwrap();
            'outer: for tx in block.get_transactions() {
                let txid_hex = HEXLOWER.encode(tx.get_id());
                for (idx, out) in tx.get_vout().iter().enumerate() {
                    // 过滤已花费的输出
                    if let Some(outs) = spent_txos.get(txid_hex.as_str()) {
                        for spend_out_idx in outs {
                            if idx.eq(spend_out_idx) {
                                continue 'outer;
                            }
                        }
                    }
                    if utxo.contains_key(txid_hex.as_str()) {
                        utxo.get_mut(txid_hex.as_str()).unwrap().push(out.clone());
                    } else {
                        utxo.insert(txid_hex.clone(), vec![out.clone()]);
                    }
                }
                if tx.is_coinbase() {
                    continue;
                }
                // 在输入中查找已花费输出
                for txin in tx.get_vin() {
                    let txid_hex = HEXLOWER.encode(txin.get_txid());
                    if spent_txos.contains_key(txid_hex.as_str()) {
                        spent_txos
                            .get_mut(txid_hex.as_str())
                            .unwrap()
                            .push(txin.get_vout());
                    } else {
                        spent_txos.insert(txid_hex, vec![txin.get_vout()]);
                    }
                }
            }
        }
        utxo
    }

    /// 从区块链中查找交易
    pub fn find_transaction(&self, txid: &[u8]) -> Option<Transaction> {
        let mut iterator = self.iterator();
        loop {
            let option = iterator.next();
            if option.is_none() {
                break;
            }
            let block = option.unwrap();
            for transaction in block.get_transactions() {
                if txid.eq(transaction.get_id()) {
                    return Some(transaction.clone());
                }
            }
        }
        None
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
        let block_tree = self.db.open_tree(BLOCKS_TREE).unwrap();
        let data = block_tree.get(self.current_hash.clone()).unwrap();
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

    #[test]
    fn test_create_blockchain() {
        let blockchain = super::Blockchain::create_blockchain();
        println!("tip = {}", blockchain.tip)
    }

    #[test]
    fn test_find_transaction() {
        let blockchain = super::Blockchain::new_blockchain();
        let trasaction = blockchain.find_transaction(
            "00aee463227e52bf2c6986033d86a2572942f9d79a1da7c4cebe790a8b8ead92".as_bytes(),
        );
        assert!(trasaction.is_none())
    }
}
