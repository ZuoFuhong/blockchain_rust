use ring::signature::{EcdsaKeyPair, KeyPair, ECDSA_P256_SHA256_FIXED_SIGNING};
use serde::{Deserialize, Serialize};

const VERSION: u8 = 0x00;
pub const ADDRESS_CHECK_SUM_LEN: usize = 4;

#[derive(Clone, Serialize, Deserialize)]
pub struct Wallet {
    pkcs8: Vec<u8>,
    public_key: Vec<u8>, // 原生的公钥
}

impl Wallet {
    /// 创建一个钱包
    pub fn new() -> Wallet {
        let pkcs8 = crate::new_key_pair();
        let key_pair =
            EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_FIXED_SIGNING, pkcs8.as_ref()).unwrap();
        let public_key = key_pair.public_key().as_ref().to_vec();
        Wallet { pkcs8, public_key }
    }

    /// 获取钱包地址
    /// 这里得到了一个真实的BTC地址，可以在 (Tokenview)[https://tokenview.com/cn/search/173EuX6KuB1EiWYEKyaQud6x91VNjkM3Vu] 查询它的余额.
    /// 不过我可以负责任地说，无论生成一个新的地址多少次，检查它的余额都是 0。这就是为什么选择一个合适的公钥加密算法是如此重要：考虑到私钥是随机数，生成
    /// 同一个数字的概率必须是尽可能地低。理想情况下，必须是低到“永远”不会重复。
    /// 另外，注意：你并不需要连接到一个比特币节点来获得一个地址。地址生成算法使用的多种开源算法可以通过很多编程语言和库实现。
    pub fn get_address(&self) -> String {
        let pub_key_hash = hash_pub_key(self.public_key.as_slice());
        let mut payload: Vec<u8> = vec![];
        payload.push(VERSION);
        payload.extend(pub_key_hash.as_slice());
        let checksum = checksum(payload.as_slice());
        payload.extend(checksum.as_slice());
        // version + pub_key_hash + checksum
        crate::base58_encode(payload.as_slice())
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.clone()
    }

    pub fn get_pkcs8(&self) -> Vec<u8> {
        self.pkcs8.clone()
    }
}

/// 计算公钥哈希
pub fn hash_pub_key(pub_key: &[u8]) -> Vec<u8> {
    let pub_key_sha256 = crate::sha256_digest(pub_key);
    crate::ripemd160_digest(pub_key_sha256.as_slice())
}

/// 计算校验和
fn checksum(payload: &[u8]) -> Vec<u8> {
    let first_sha = crate::sha256_digest(payload);
    let second_sha = crate::sha256_digest(first_sha.as_slice());
    second_sha[0..ADDRESS_CHECK_SUM_LEN].to_vec()
}

/// 验证地址有效
pub fn validate_address(address: &str) -> bool {
    let payload = crate::base58_decode(address);
    let actual_checksum = payload[payload.len() - ADDRESS_CHECK_SUM_LEN..].to_vec();
    let version = payload[0];
    let pub_key_hash = payload[1..payload.len() - ADDRESS_CHECK_SUM_LEN].to_vec();

    let mut target_vec = vec![];
    target_vec.push(version);
    target_vec.extend(pub_key_hash);
    let target_checksum = checksum(target_vec.as_slice());
    actual_checksum.eq(target_checksum.as_slice())
}

/// 通过公钥哈希计算地址
pub fn calc_address(pub_hash_key: &[u8]) -> String {
    let mut payload: Vec<u8> = vec![];
    payload.push(VERSION);
    payload.extend(pub_hash_key);
    let checksum = checksum(payload.as_slice());
    payload.extend(checksum.as_slice());
    crate::base58_encode(payload.as_slice())
}

#[cfg(test)]
mod tests {
    use crate::wallet::validate_address;

    #[test]
    pub fn test_get_address() {
        let wallet = crate::Wallet::new();
        let address = wallet.get_address();
        println!("The address is {}", address)
    }

    #[test]
    pub fn test_validate_address() {
        // BTC 创世块：1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
        let valid = validate_address("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
        println!("The address valid is {}", valid);

        // 1CwipMiukwDsmhQdVG2u7gs3BasumzfAD5
        let valid = validate_address("1CwipMiukwDsmhQdVG2u7gs3BasumzfAD5");
        println!("The address valid is {}", valid);
    }
}
