use crate::Transaction;
use data_encoding::HEXLOWER;
use std::collections::HashMap;
use std::sync::RwLock;

/// 交易内存池 ( K -> txid_hex, V => Transaction )
pub struct MemoryPool {
    inner: RwLock<HashMap<String, Transaction>>,
}

impl MemoryPool {
    pub fn new() -> MemoryPool {
        MemoryPool {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn containes(&self, txid_hex: &str) -> bool {
        self.inner.read().unwrap().contains_key(txid_hex)
    }

    pub fn add(&self, tx: Transaction) {
        let txid_hex = HEXLOWER.encode(tx.get_id());
        self.inner.write().unwrap().insert(txid_hex, tx);
    }

    pub fn get(&self, txid_hex: &str) -> Option<Transaction> {
        if let Some(tx) = self.inner.read().unwrap().get(txid_hex) {
            return Some(tx.clone());
        }
        None
    }

    pub fn remove(&self, txid_hex: &str) {
        let mut inner = self.inner.write().unwrap();
        inner.remove(txid_hex);
    }

    pub fn get_all(&self) -> Vec<Transaction> {
        let inner = self.inner.read().unwrap();
        let mut txs = vec![];
        for (_, v) in inner.iter() {
            txs.push(v.clone());
        }
        return txs;
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
}

/// 传输中的块, 用于来跟踪已下载的块, 这能够实现从不同的节点下载块.
pub struct BlockInTransit {
    inner: RwLock<Vec<Vec<u8>>>,
}

impl BlockInTransit {
    pub fn new() -> BlockInTransit {
        BlockInTransit {
            inner: RwLock::new(vec![]),
        }
    }

    pub fn add_blocks(&self, blocks: &[Vec<u8>]) {
        let mut inner = self.inner.write().unwrap();
        for hash in blocks {
            inner.push(hash.to_vec());
        }
    }

    pub fn first(&self) -> Option<Vec<u8>> {
        let inner = self.inner.read().unwrap();
        if let Some(block_hash) = inner.first() {
            return Some(block_hash.to_vec());
        }
        None
    }

    pub fn remove(&self, block_hash: &[u8]) {
        let mut inner = self.inner.write().unwrap();
        if let Some(idx) = inner.iter().position(|x| x.eq(block_hash)) {
            inner.remove(idx);
        }
    }

    pub fn len(&self) -> usize {
        self.inner.read().unwrap().len()
    }
}

#[cfg(test)]
mod tests {
    use super::{BlockInTransit, MemoryPool};
    use crate::Transaction;
    use data_encoding::HEXLOWER;

    #[test]
    fn test_memory_pool() {
        let pool = MemoryPool::new();
        let tx = Transaction::new_coinbase_tx("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
        let txid_hex = HEXLOWER.encode(tx.get_id());
        pool.add(tx);
        let option = pool.get(txid_hex.as_str());
        assert!(option.is_some());

        pool.remove(txid_hex.as_str());
        let option = pool.get(txid_hex.as_str());
        assert!(option.is_none());
    }

    #[test]
    fn test_blocks_in_transit() {
        let mut block_hashs = vec![];
        block_hashs.push("a123".as_bytes().to_vec());
        block_hashs.push("b123".as_bytes().to_vec());
        block_hashs.push("c123".as_bytes().to_vec());

        let transit = BlockInTransit::new();
        transit.add_blocks(&block_hashs);
        assert_eq!(transit.first().unwrap(), "a123".as_bytes().to_vec());

        transit.remove("a123".as_bytes());
        assert_eq!(transit.first().unwrap(), "b123".as_bytes().to_vec());
    }
}
