use crate::Block;
use std::borrow::Borrow;

pub struct Blockchain {
    blocks: Vec<Block>,
}

impl Blockchain {
    /// 创建区块链
    pub fn new_blockchain() -> Blockchain {
        let mut blocks = Vec::new();
        let genesis_block = Block::new_genesis_block();
        blocks.push(genesis_block);
        Blockchain { blocks }
    }

    /// 增加区块
    pub fn add_block(&mut self, data: String) {
        let pre_block_hash = self.blocks[self.blocks.len() - 1].borrow().get_hash();
        let new_block = Block::new_block(pre_block_hash, data);
        self.blocks.push(new_block);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    pub fn test_blockchain() {
        let mut blockchain = super::Blockchain::new_blockchain();
        blockchain.add_block(String::from("Send 1 BTC to Mars"));
        for block in blockchain.blocks {
            println!("Pre block hash: {}", block.get_pre_block_hash());
            println!("Cur block hash: {}", block.get_hash());
            println!("Data: {}", block.get_data());
            println!("Timestamp: {}\n", block.get_timestamp());
        }
    }
}
