use data_encoding::HEXLOWER;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 挖矿奖励金
const SUBSIDY: i32 = 10;

/// 交易输入
#[derive(Clone, Serialize, Deserialize)]
pub struct TXInput {
    txid: Vec<u8>,      // 一个交易输入引用了前一笔交易的一个输出，ID表明是之前的哪一笔交易
    vout: i32,          // 输出的索引
    script_sig: String, // 提供解锁输出的数据
}

impl TXInput {
    pub fn can_unlock_output_with(&self, unlocking_data: &str) -> bool {
        self.script_sig.eq(unlocking_data)
    }

    pub fn get_txid(&self) -> Vec<u8> {
        self.txid.clone()
    }

    pub fn get_vout(&self) -> i32 {
        self.vout
    }

    pub fn get_script_sig(&self) -> String {
        self.script_sig.clone()
    }
}

/// 交易输出
#[derive(Clone, Serialize, Deserialize)]
pub struct TXOutput {
    value: i32,             // 币的数量
    script_pub_key: String, // 对输出进行锁定
}

impl TXOutput {
    /// 新建输出
    pub fn new(value: i32, address: String) -> TXOutput {
        TXOutput {
            value,
            script_pub_key: address,
        }
    }

    pub fn get_value(&self) -> i32 {
        self.value
    }

    pub fn get_script_pub_key(&self) -> String {
        self.script_pub_key.clone()
    }

    pub fn can_be_unlocked_with(&self, unlocking_data: &str) -> bool {
        self.script_pub_key.eq(unlocking_data)
    }
}

/// 交易
#[derive(Clone, Serialize, Deserialize)]
pub struct Transaction {
    id: Vec<u8>,         // 交易ID
    vin: Vec<TXInput>,   // 输入
    vout: Vec<TXOutput>, // 输出
}

impl Transaction {
    /// 创建一个 coinbase 交易，该没有输入，只有一个输出
    pub fn new_coinbase_tx(to: String, mut data: String) -> Transaction {
        if data.len() == 0 {
            data = format!("Reward to {}", to)
        }
        let txin = TXInput {
            txid: vec![],
            vout: -1,
            script_sig: data,
        };
        let txout = TXOutput {
            value: SUBSIDY,
            script_pub_key: to,
        };
        let mut tx = Transaction {
            id: vec![],
            vin: vec![txin],
            vout: vec![txout],
        };
        tx.set_id();
        return tx;
    }

    /// 创建一笔 UTXO 的交易
    pub fn new_utxo_transaction(
        from: String,
        to: String,
        amount: i32,
        unspent_transaction: Vec<Transaction>,
    ) -> Transaction {
        // 找到足够的未花费输出
        let mut accumulated = 0;
        let mut valid_outputs: HashMap<String, Vec<i32>> = HashMap::new();
        'outer: for tx in unspent_transaction {
            let txid_hex = HEXLOWER.encode(tx.id.as_slice());
            for idx in 0..tx.vout.len() {
                let txout = tx.vout[idx].clone();
                if txout.can_be_unlocked_with(from.as_str()) {
                    accumulated += txout.value;
                    if valid_outputs.contains_key(txid_hex.as_str()) {
                        valid_outputs
                            .get_mut(txid_hex.as_str())
                            .unwrap()
                            .push(idx as i32);
                    } else {
                        valid_outputs.insert(txid_hex.clone(), vec![idx as i32]);
                    }
                    if accumulated >= amount {
                        break 'outer;
                    }
                }
            }
        }
        if accumulated < amount {
            panic!("Error: Not enough funds")
        }
        // 交易的输入
        let mut inputs = vec![];
        for (txid_hex, outs) in valid_outputs {
            let txid = HEXLOWER.decode(txid_hex.as_bytes()).unwrap();
            for out in outs {
                let input = TXInput {
                    txid: txid.clone(),       // 上一笔交易的ID
                    vout: out,                // 输出的索引
                    script_sig: from.clone(), // from: 支出币
                };
                inputs.push(input);
            }
        }
        // 交易的输出
        let mut outputs = vec![TXOutput::new(amount, to.clone())];
        // 如果 UTXO 总数超过所需，则产生找零
        if accumulated > amount {
            outputs.push(TXOutput::new(accumulated - amount, from)) // to: 币收入
        }
        let mut tx = Transaction {
            id: vec![],
            vin: inputs,
            vout: outputs,
        };
        // 生成交易ID
        tx.set_id();
        return tx;
    }

    /// 判断是否是 coinbase 交易
    pub fn is_coinbase(&self) -> bool {
        return self.vin.len() == 1 && self.vin[0].txid.len() == 0 && self.vin[0].vout == -1;
    }

    fn set_id(&mut self) {
        let data = bincode::serialize(self).unwrap();
        self.id = crate::sha256_digest(data.as_slice());
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
    use crate::Transaction;
    use data_encoding::HEXLOWER;

    #[test]
    fn new_coinbase_tx() {
        let tx = Transaction::new_coinbase_tx(String::from("mars"), String::from("miko"));
        let txid_hex = HEXLOWER.encode(tx.get_id().as_slice());
        println!("txid = {}", txid_hex);
    }
}
