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
    pub fn new_block(pre_block_hash: String, transactions: Vec<Transaction>) -> Block {
        let mut block = Block {
            timestamp: crate::current_timestamp(),
            pre_block_hash,
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
        return Block::new_block(String::from("None"), vec![transaction]);
    }

    /// 计算区块里所有交易的哈希
    pub fn hash_transactions(&self) -> Vec<u8> {
        let mut txhashs = vec![];
        for transaction in self.transactions.clone() {
            let txid = transaction.get_id();
            txhashs.extend(txid.as_slice());
        }
        crate::sha256_digest(txhashs.as_slice())
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.transactions.clone()
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
    use data_encoding::HEXLOWER;

    #[test]
    fn test_sha256_digest() {
        // sha256 会产生256位的哈希值，作为消息的摘要。这个摘要相当于一个32个字节的数组，通常有一个长度为64的16进制
        // 字符串表示，其中一个字节等于8位，一个16进制的字符长度为4位。
        let digest = crate::sha256_digest("hello".as_bytes());
        // 16进制编码输出
        let hex_digest = HEXLOWER.encode(digest.as_slice());
        println!("SHA-256 digest is {}", hex_digest)
    }

    #[test]
    fn test_new_block() {
        let block = Block::new_block(
            String::from("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"),
            vec![],
        );
        println!("new block hash is {}", block.hash)
    }

    #[test]
    fn test_block_serialize() {
        let tx =
            Transaction::new_coinbase_tx(String::from("Genesis"), String::from("Genesis data"));
        let block = Block::new_block(
            String::from("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"),
            vec![tx],
        );
        let bytes = bincode::serialize(&block).unwrap();
        let desc_block = Block::deserialize(&bytes[..]);
        assert_eq!(block.hash, desc_block.hash)
    }
}
