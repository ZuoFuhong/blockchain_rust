use data_encoding::HEXLOWER;
use ring::digest::{Context, SHA256};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Block {
    timestamp: i64,         // 区块时间戳
    pre_block_hash: String, // 上一区块的哈希值
    hash: String,           // 当前区块的哈希值
    data: String,           // 区块数据
}

impl Block {
    /// 新建一个区块
    pub fn new_block(pre_block_hash: String, data: String) -> Block {
        let timestamp = current_timestamp();
        let hash = caculate_hash(timestamp, pre_block_hash.clone(), data.clone());
        Block {
            timestamp,
            pre_block_hash,
            hash,
            data,
        }
    }

    /// 生成创世块
    pub fn new_genesis_block() -> Block {
        return Block::new_block(String::new(), String::from("Genesis Block"));
    }

    pub fn get_pre_block_hash(&self) -> String {
        self.pre_block_hash.clone()
    }

    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn get_data(&self) -> String {
        self.data.clone()
    }

    pub fn get_timestamp(&self) -> i64 {
        self.timestamp
    }
}

/// 获取当前时间戳，单位：ms
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
}

/// 计算区块哈希值
fn caculate_hash(timestamp: i64, pre_block_hash: String, data: String) -> String {
    let block_data = format!("{}{}{}", timestamp, pre_block_hash, data);
    sha256_digest(block_data)
}

fn sha256_digest(data: String) -> String {
    let mut context = Context::new(&SHA256);
    context.update(data.as_bytes());
    let digest = context.finish();
    return HEXLOWER.encode(digest.as_ref());
}

#[cfg(test)]
mod tests {
    use super::Block;

    #[test]
    fn test_new_block() {
        let block = Block::new_block(
            String::from("2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"),
            String::from("ABC"),
        );
        println!("new block hash is {}", block.hash)
    }

    #[test]
    fn test_sha256_digest() {
        let digest = super::sha256_digest(String::from("hello"));
        println!("SHA-256 digest is {}", digest)
    }
}
