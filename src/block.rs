use crate::{ProofOfWork, Transaction};
use serde::{Deserialize, Serialize};
use sled::IVec;

#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    timestamp: i64,                 // 区块时间戳
    pre_block_hash: String,         // 上一区块的哈希值
    hash: String,                   // 当前区块的哈希值
    transactions: Vec<Transaction>, // 交易数据
    nonce: i64,                     // 计数器
}

impl Block {
    /// 新建一个区块
    pub fn new_block(pre_block_hash: &str, transactions: Vec<Transaction>) -> Block {
        let mut block = Block {
            timestamp: crate::current_timestamp(),
            pre_block_hash: String::from(pre_block_hash),
            hash: String::new(),
            transactions,
            nonce: 0,
        };
        // 挖矿计算哈希
        let pow = ProofOfWork::new_proof_of_work(block.clone());
        let (nonce, hash) = pow.run();
        block.nonce = nonce;
        block.hash = hash;
        return block;
    }

    /// 从字节数组反序列化
    pub fn deserialize(bytes: &[u8]) -> Block {
        bincode::deserialize(bytes).unwrap()
    }

    /// 生成创世块
    pub fn generate_genesis_block(transaction: Transaction) -> Block {
        return Block::new_block("None", vec![transaction]);
    }

    /// 计算区块里所有交易的哈希
    pub fn hash_transactions(&self) -> Vec<u8> {
        let mut txhashs = vec![];
        for transaction in &self.transactions {
            txhashs.extend(transaction.get_id());
        }
        crate::sha256_digest(txhashs.as_slice())
    }

    pub fn get_transactions(&self) -> &[Transaction] {
        self.transactions.as_slice()
    }

    pub fn get_pre_block_hash(&self) -> String {
        self.pre_block_hash.clone()
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}

impl From<Block> for IVec {
    fn from(b: Block) -> Self {
        let bytes = bincode::serialize(&b).unwrap();
        Self::from(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::Block;
    use crate::Transaction;

    #[test]
    fn test_new_block() {
        let block = Block::new_block(
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
            vec![],
        );
        println!("new block hash is {}", block.hash)
    }

    #[test]
    fn test_block_serialize() {
        let tx = Transaction::new_coinbase_tx("Genesis");
        let block = Block::new_block(
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
            vec![tx],
        );
        let bytes = bincode::serialize(&block).unwrap();
        let desc_block = Block::deserialize(&bytes[..]);
        assert_eq!(block.hash, desc_block.hash)
    }
}
