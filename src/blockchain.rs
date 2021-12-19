use crate::Block;
use sled::Db;
use std::env::current_dir;

const TIP_BLOCK_HASH_KEY: &str = "tip_block_hash";

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
            let block = Block::new_genesis_block();
            let block_hash = block.get_hash();
            let _ = db.insert(block_hash.clone(), block);
            let _ = db.insert(TIP_BLOCK_HASH_KEY, block_hash.as_bytes().to_vec());
            tip = block_hash;
        } else {
            tip = String::from_utf8(data.unwrap().to_vec()).unwrap();
        }
        Blockchain { tip, db }
    }

    /// 增加区块
    pub fn add_block(&mut self, data: String) {
        let block = Block::new_block(self.tip.clone(), data);
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
    use std::env::current_dir;

    #[test]
    fn test_blockchain() {
        let mut blockchain = super::Blockchain::new_blockchain();
        blockchain.add_block(String::from("Send 1 BTC to Mars"));
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
