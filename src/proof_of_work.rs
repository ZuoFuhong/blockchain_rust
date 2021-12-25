use crate::Block;
use data_encoding::HEXLOWER;
use num_bigint::{BigInt, Sign};
use std::borrow::Borrow;
use std::ops::ShlAssign;

pub struct ProofOfWork {
    block: Block,
    target: BigInt,
}

/// 难度值，这里表示哈希的前20位必须是0
const TARGET_BITS: i32 = 20;
/// 限制 nonce 避免整型溢出
const MAX_NONCE: i64 = i64::MAX;

impl ProofOfWork {
    pub fn new_proof_of_work(block: Block) -> ProofOfWork {
        let mut target = BigInt::from(1);
        // target 等于 1 左移 256 - TARGET_BITS 位
        target.shl_assign(256 - TARGET_BITS);
        ProofOfWork { block, target }
    }

    /// 工作量证明用到的数据
    fn prepare_data(&self, nonce: i64) -> Vec<u8> {
        let pre_block_hash = self.block.get_pre_block_hash();
        let transactions_hash = self.block.hash_transactions();
        let timestamp = self.block.get_timestamp();
        let mut data_bytes = vec![];
        data_bytes.extend(pre_block_hash.as_bytes());
        data_bytes.extend(transactions_hash);
        data_bytes.extend(timestamp.to_be_bytes());
        data_bytes.extend(TARGET_BITS.to_be_bytes());
        data_bytes.extend(nonce.to_be_bytes());
        return data_bytes;
    }

    /// 工作量证明的核心就是寻找有效的哈希
    pub fn run(&self) -> (i64, String) {
        let mut nonce = 0;
        let mut hash = Vec::new();
        println!("Mining the block");
        while nonce < MAX_NONCE {
            let data = self.prepare_data(nonce);
            hash = crate::sha256_digest(data.as_slice());
            let hash_int = BigInt::from_bytes_be(Sign::Plus, hash.as_slice());

            // 1.在比特币中，当一个块被挖出来以后，“target bits” 代表了区块头里存储的难度，也就是开头有多少个 0。
            // 2.这里的 20 指的是算出来的哈希前 20 位必须是 0，如果用 16 进制表示，就是前 5 位必须是 0，这一点从
            //   最后的输出可以看出来。
            //   例如：target 16进制输出是 0000100000000000000000000000000000000000000000000000000000000000
            //   目前我们并不会实现一个动态调整目标的算法，所以将难度定义为一个全局的常量即可。
            // 3.将哈希与目标数 target 进行比较：先把哈希转换成一个大整数，然后检测它是否小于目标，小就是有效的，反之无效。
            if hash_int.lt(self.target.borrow()) {
                println!("{}", HEXLOWER.encode(hash.as_slice()));
                break;
            } else {
                nonce += 1;
            }
        }
        println!();
        return (nonce, HEXLOWER.encode(hash.as_slice()));
    }
}

#[cfg(test)]
mod tests {
    use super::TARGET_BITS;
    use data_encoding::HEXLOWER;
    use num_bigint::BigInt;
    use std::ops::ShlAssign;

    #[test]
    fn test_bigint_from_bytes() {
        let a = BigInt::from(256); // 0 ~ 255
        let (s, vec) = a.to_bytes_be();
        println!("{:?}, {:?}", s, vec);

        // big-endian
        let b = BigInt::from_signed_bytes_be(vec.as_slice());
        println!("{}", b)
    }

    #[test]
    fn test_target_bits() {
        let mut target = BigInt::from(1);
        target.shl_assign(256 - TARGET_BITS);
        println!("{}", target); // output: 6901746346790563787434755862277025452451108972170386555162524223799296

        // 16进制输出, 大端序
        let (_, vec) = target.to_bytes_be();
        let target_hex = HEXLOWER.encode(vec.as_slice());
        println!("{}", target_hex) // output: 100000000000000000000000000000000000000000000000000000000000
    }
}
