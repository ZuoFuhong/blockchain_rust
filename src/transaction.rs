use crate::wallet::hash_pub_key;
use crate::{base58_decode, wallet, Blockchain, Wallets};
use data_encoding::HEXLOWER;
use serde::{Deserialize, Serialize};

/// 挖矿奖励金
const SUBSIDY: i32 = 10;

/// 交易输入
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct TXInput {
    txid: Vec<u8>,      // 一个交易输入引用了前一笔交易的一个输出，ID表明是之前的哪一笔交易
    vout: usize,        // 输出的索引
    signature: Vec<u8>, // 签名
    pub_key: Vec<u8>,   // 原生的公钥
}

impl TXInput {
    /// 创建一个输入
    pub fn new(txid: Vec<u8>, vout: usize) -> TXInput {
        TXInput {
            txid,
            vout,
            signature: vec![],
            pub_key: vec![],
        }
    }

    pub fn get_txid(&self) -> Vec<u8> {
        self.txid.clone()
    }

    pub fn get_vout(&self) -> usize {
        self.vout
    }

    pub fn get_pub_key(&self) -> Vec<u8> {
        self.pub_key.clone()
    }

    /// 检查输入使用了指定密钥来解锁一个输出
    pub fn uses_key(&self, pub_key_hash: &[u8]) -> bool {
        let locking_hash = wallet::hash_pub_key(self.pub_key.as_slice());
        return locking_hash.eq(pub_key_hash);
    }
}

/// 交易输出
#[derive(Clone, Serialize, Deserialize)]
pub struct TXOutput {
    value: i32,            // 币的数量
    pub_key_hash: Vec<u8>, // 公钥哈希
}

impl TXOutput {
    /// 创建一个输出
    pub fn new(value: i32, address: &str) -> TXOutput {
        let mut output = TXOutput {
            value,
            pub_key_hash: vec![],
        };
        output.lock(address);
        return output;
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }

    pub fn get_pub_key_hash(&self) -> Vec<u8> {
        self.pub_key_hash.clone()
    }

    fn lock(&mut self, address: &str) {
        let payload = base58_decode(address);
        let pub_key_hash = payload[1..payload.len() - wallet::ADDRESS_CHECK_SUM_LEN].to_vec();
        self.pub_key_hash = pub_key_hash;
    }

    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash.eq(pub_key_hash)
    }
}

/// 交易
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Transaction {
    id: Vec<u8>,         // 交易ID
    vin: Vec<TXInput>,   // 输入
    vout: Vec<TXOutput>, // 输出
}

impl Transaction {
    /// 创建一个 coinbase 交易，该没有输入，只有一个输出
    pub fn new_coinbase_tx(to: &str) -> Transaction {
        let txin = TXInput::default();
        let txout = TXOutput::new(SUBSIDY, to);
        let mut tx = Transaction {
            id: vec![],
            vin: vec![txin],
            vout: vec![txout],
        };
        tx.id = tx.hash();
        return tx;
    }

    /// 创建一笔 UTXO 的交易
    pub fn new_utxo_transaction(
        from: &str,
        to: &str,
        amount: i32,
        blockchain: &Blockchain,
    ) -> Transaction {
        // 1.查找钱包
        let wallet = Wallets::new()
            .get_wallet(from)
            .expect("unable to found wallet");
        let public_key_hash = hash_pub_key(wallet.get_public_key().as_slice());
        // 2.找到足够的未花费输出
        let (accumulated, valid_outputs) =
            blockchain.find_spendable_outputs(public_key_hash.as_slice(), amount);
        if accumulated < amount {
            panic!("Error: Not enough funds")
        }
        // 3.交易数据
        // 3.1.交易的输入
        let mut inputs = vec![];
        for (txid_hex, outs) in valid_outputs {
            let txid = HEXLOWER.decode(txid_hex.as_bytes()).unwrap();
            for out in outs {
                let input = TXInput {
                    txid: txid.clone(), // 上一笔交易的ID
                    vout: out,          // 输出的索引
                    signature: vec![],
                    pub_key: wallet.get_public_key(),
                };
                inputs.push(input);
            }
        }
        // 3.2.交易的输出
        let mut outputs = vec![TXOutput::new(amount, to)];
        // 如果 UTXO 总数超过所需，则产生找零
        if accumulated > amount {
            outputs.push(TXOutput::new(accumulated - amount, from)) // to: 币收入
        }
        // 4.生成交易
        let mut tx = Transaction {
            id: vec![],
            vin: inputs,
            vout: outputs,
        };
        // 生成交易ID
        tx.id = tx.hash();
        // 5.交易中的 TXInput 签名
        tx.sign(blockchain, wallet.get_pkcs8());
        return tx;
    }

    /// 创建一个修剪后的交易副本
    fn trimmed_copy(&self) -> Transaction {
        let mut inputs = vec![];
        let mut outputs = vec![];
        for input in &self.vin {
            let txinput = TXInput::new(input.get_txid(), input.get_vout());
            inputs.push(txinput);
        }
        for output in &self.vout {
            outputs.push(output.clone());
        }
        Transaction {
            id: self.id.clone(),
            vin: inputs,
            vout: outputs,
        }
    }

    /// 对交易的每个输入进行签名
    fn sign(&mut self, blockchain: &Blockchain, pkcs8: Vec<u8>) {
        let mut tx_copy = self.trimmed_copy();

        for (idx, vin) in self.vin.iter_mut().enumerate() {
            // 查找输入引用的交易
            let prev_tx_option = blockchain.find_transaction(vin.get_txid().as_slice());
            if prev_tx_option.is_none() {
                panic!("ERROR: Previous transaction is not correct")
            }
            let prev_tx = prev_tx_option.unwrap();
            tx_copy.vin[idx].signature = vec![];
            tx_copy.vin[idx].pub_key = prev_tx.vout[vin.vout].pub_key_hash.clone();
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[idx].pub_key = vec![];

            // 使用私钥对数据签名
            let tx_bytes = bincode::serialize(&tx_copy).expect("unable to serialize transaction");
            let signature =
                crate::ecdsa_p256_sha256_sign_digest(pkcs8.as_slice(), tx_bytes.as_slice());
            vin.signature = signature;
        }
    }

    /// 对交易的每个输入进行签名验证
    pub fn verify(&self, blockchain: &Blockchain) -> bool {
        if self.is_coinbase() {
            return true;
        }
        let mut tx_copy = self.trimmed_copy();
        for (idx, vin) in self.vin.iter().enumerate() {
            let prev_tx_option = blockchain.find_transaction(vin.get_txid().as_slice());
            if prev_tx_option.is_none() {
                panic!("ERROR: Previous transaction is not correct")
            }
            let prev_tx = prev_tx_option.unwrap();
            tx_copy.vin[idx].signature = vec![];
            tx_copy.vin[idx].pub_key = prev_tx.vout[vin.vout].pub_key_hash.clone();
            tx_copy.id = tx_copy.hash();
            tx_copy.vin[idx].pub_key = vec![];

            // 使用公钥验证签名
            let tx_bytes = bincode::serialize(&tx_copy).expect("unable to serialize transaction");
            let verify = crate::ecdsa_p256_sha256_sign_verify(
                vin.pub_key.as_slice(),
                vin.signature.as_slice(),
                tx_bytes.as_slice(),
            );
            if !verify {
                return false;
            }
        }
        true
    }

    /// 判断是否是 coinbase 交易
    pub fn is_coinbase(&self) -> bool {
        return self.vin.len() == 1 && self.vin[0].txid.len() == 0 && self.vin[0].vout == 0;
    }

    /// 生成交易的哈希
    fn hash(&mut self) -> Vec<u8> {
        let tx_copy = Transaction {
            id: vec![],
            vin: self.vin.clone(),
            vout: self.vout.clone(),
        };
        let data = bincode::serialize(&tx_copy).unwrap();
        crate::sha256_digest(data.as_slice())
    }

    pub fn get_id(&self) -> Vec<u8> {
        return self.id.clone();
    }

    pub fn get_vin(&self) -> Vec<TXInput> {
        self.vin.clone()
    }

    pub fn get_vout(&self) -> Vec<TXOutput> {
        self.vout.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Blockchain, Transaction};
    use data_encoding::HEXLOWER;

    #[test]
    fn new_coinbase_tx() {
        let tx = Transaction::new_coinbase_tx("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
        let txid_hex = HEXLOWER.encode(tx.get_id().as_slice());
        println!("txid = {}", txid_hex);
    }

    #[test]
    fn new_utxo_transaction() {
        let blockchain = Blockchain::new_blockchain();
        let tx = Transaction::new_utxo_transaction(
            "1CjtQaWmX1SSbB3ySoFYCFCVpQbTsejpa7",
            "1CjtQaWmX1SSbB3ySoFYCFCVpQbTsejpa7",
            5,
            &blockchain,
        );
        let txid_hex = HEXLOWER.encode(tx.get_id().as_slice());
        println!("txid = {}", txid_hex);
    }
}
