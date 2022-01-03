use crate::{
    Block, BlockInTransit, Blockchain, MemoryPool, Nodes, Transaction, UTXOSet, GLOBAL_CONFIG,
};
use data_encoding::HEXLOWER;
use log::{error, info};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::error::Error;
use std::io::{BufReader, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

/// 版本硬编码
const NODE_VERSION: usize = 1;

/// 中心节点硬编码
pub const CENTERAL_NODE: &str = "127.0.0.1:2001";

/// 内存池中的交易到达阈值, 触发矿工挖新区块
pub const TRANSACTION_THRESHOLD: usize = 2;

/// 全网的节点地址
static GLOBAL_NODES: Lazy<Nodes> = Lazy::new(|| {
    let nodes = Nodes::new();
    // 记录中心地址
    nodes.add_node(String::from(CENTERAL_NODE));
    return nodes;
});

/// 交易内存池
static GLOBAL_MEMORY_POOL: Lazy<MemoryPool> = Lazy::new(|| MemoryPool::new());

/// 传输中的Block, 用于来跟踪已下载的块, 这能够实现从不同的节点下载块
static GLOBAL_BLOCKS_IN_TRANSIT: Lazy<BlockInTransit> = Lazy::new(|| BlockInTransit::new());

/// 网络写超时
const TCP_WRITE_TIMEOUT: u64 = 1000;

pub struct Server {
    blockchain: Blockchain,
}

impl Server {
    pub fn new(blockchain: Blockchain) -> Server {
        Server { blockchain }
    }

    pub fn run(&self, addr: &str) {
        let listener = TcpListener::bind(addr).unwrap();

        // 发送 version 握手
        if addr.eq(CENTERAL_NODE) == false {
            let best_height = self.blockchain.get_best_height();
            info!("send sersion best_height: {}", best_height);
            send_version(CENTERAL_NODE, best_height);
        }
        info!("Start node server on {}", addr);
        for stream in listener.incoming() {
            let blockchain = self.blockchain.clone();
            thread::spawn(|| match stream {
                Ok(stream) => {
                    if let Err(e) = serve(blockchain, stream) {
                        error!("Error on serving client: {}", e);
                    }
                }
                Err(e) => {
                    error!("Connection failed: {}", e);
                }
            });
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum OpType {
    Tx,
    Block,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Package {
    Block {
        addr_from: String,
        block: Vec<u8>,
    },
    GetBlocks {
        addr_from: String,
    },
    GetData {
        addr_from: String,
        op_type: OpType,
        id: Vec<u8>,
    },
    Inv {
        addr_from: String,
        op_type: OpType,
        items: Vec<Vec<u8>>,
    },
    Tx {
        addr_from: String,
        transaction: Vec<u8>,
    },
    Version {
        addr_from: String,
        version: usize,
        best_height: usize,
    },
}

fn send_get_data(addr: &str, op_type: OpType, block_hash: &[u8]) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::GetData {
            addr_from: node_addr,
            op_type,
            id: block_hash.to_vec(),
        },
    );
}

fn send_inv(addr: &str, op_type: OpType, blocks: &[Vec<u8>]) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Inv {
            addr_from: node_addr,
            op_type,
            items: blocks.to_vec(),
        },
    );
}

fn send_block(addr: &str, block: &Block) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Block {
            addr_from: node_addr,
            block: block.serialize(),
        },
    );
}

pub fn send_tx(addr: &str, tx: &Transaction) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Tx {
            addr_from: node_addr,
            transaction: tx.serialize(),
        },
    );
}

fn send_version(addr: &str, height: usize) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::Version {
            addr_from: node_addr,
            version: NODE_VERSION,
            best_height: height,
        },
    );
}

fn send_get_blocks(addr: &str) {
    let socket_addr = addr.parse().unwrap();
    let node_addr = GLOBAL_CONFIG.get_node_addr().parse().unwrap();
    send_data(
        socket_addr,
        Package::GetBlocks {
            addr_from: node_addr,
        },
    );
}

fn serve(blockchain: Blockchain, stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let peer_addr = stream.peer_addr()?;
    let reader = BufReader::new(&stream);
    let pkg_reader = Deserializer::from_reader(reader).into_iter::<Package>();
    for pkg in pkg_reader {
        let pkg = pkg?;
        info!("Receive request from {}: {:?}", peer_addr, pkg);
        match pkg {
            Package::Block { addr_from, block } => {
                let block = Block::deserialize(block.as_slice());
                blockchain.add_block(&block);
                info!("Added block {}", block.get_hash());

                if GLOBAL_BLOCKS_IN_TRANSIT.len() > 0 {
                    // 继续下载区块
                    let block_hash = GLOBAL_BLOCKS_IN_TRANSIT.first().unwrap();
                    send_get_data(addr_from.as_str(), OpType::Block, &block_hash);
                    // 从下载列表中移除
                    GLOBAL_BLOCKS_IN_TRANSIT.remove(block_hash.as_slice());
                } else {
                    // 区块全部下载后，再重建索引
                    let utxo_set = UTXOSet::new(blockchain.clone());
                    utxo_set.reindex();
                }
            }
            Package::GetBlocks { addr_from } => {
                let blocks = blockchain.get_block_hashes();
                send_inv(addr_from.as_str(), OpType::Block, &blocks);
            }
            Package::GetData {
                addr_from,
                op_type,
                id,
            } => match op_type {
                OpType::Block => {
                    if let Some(block) = blockchain.get_block(id.as_slice()) {
                        send_block(addr_from.as_str(), &block);
                    }
                }
                OpType::Tx => {
                    let txid_hex = HEXLOWER.encode(id.as_slice());
                    if let Some(tx) = GLOBAL_MEMORY_POOL.get(txid_hex.as_str()) {
                        send_tx(addr_from.as_str(), &tx);
                    }
                }
            },
            Package::Inv {
                addr_from,
                op_type,
                items,
            } => match op_type {
                // 两种触发情况：
                //  1. 当 version 消息检查到区块高度落后，会收到全量的 block hash 列表。
                //  2. 矿工挖出新的区块后，会将新区块的 hash 广播给所有节点。
                OpType::Block => {
                    // 初始启动才会触发，不可能有存量数据
                    GLOBAL_BLOCKS_IN_TRANSIT.add_blocks(items.as_slice());

                    // 下载一个区块
                    let block_hash = items.get(0).unwrap();
                    send_get_data(addr_from.as_str(), OpType::Block, block_hash);
                    // 从下载列表中移除
                    GLOBAL_BLOCKS_IN_TRANSIT.remove(block_hash);
                }
                OpType::Tx => {
                    let txid = items.get(0).unwrap();
                    let txid_hex = HEXLOWER.encode(txid);

                    // 检查交易池，不包含交易则下载
                    if GLOBAL_MEMORY_POOL.containes(txid_hex.as_str()) == false {
                        send_get_data(addr_from.as_str(), OpType::Tx, txid);
                    }
                }
            },
            Package::Tx {
                addr_from,
                transaction,
            } => {
                // 记录交易到内存池
                let tx = Transaction::deserialize(transaction.as_slice());
                let txid = tx.get_id_bytes();
                GLOBAL_MEMORY_POOL.add(tx);

                let node_addr = GLOBAL_CONFIG.get_node_addr();
                // 中心节点（广播交易）
                if node_addr.eq(CENTERAL_NODE) {
                    let nodes = GLOBAL_NODES.get_nodes();
                    for node in &nodes {
                        if node_addr.eq(node.get_addr().as_str()) {
                            continue;
                        }
                        if addr_from.eq(node.get_addr().as_str()) {
                            continue;
                        }
                        send_inv(node.get_addr().as_str(), OpType::Tx, &vec![txid.clone()])
                    }
                }
                // 矿工节点（内存池中的交易到达一定数量，挖出新区块）
                if GLOBAL_MEMORY_POOL.len() >= TRANSACTION_THRESHOLD && GLOBAL_CONFIG.is_miner() {
                    // 挖矿奖励
                    let mining_address = GLOBAL_CONFIG.get_mining_addr().unwrap();
                    let coinbase_tx = Transaction::new_coinbase_tx(mining_address.as_str());
                    let mut txs = GLOBAL_MEMORY_POOL.get_all();
                    txs.push(coinbase_tx);

                    // 挖区块
                    let new_block = blockchain.mine_block(&txs);
                    let utxo_set = UTXOSet::new(blockchain.clone());
                    utxo_set.reindex();
                    info!("New block {} is mined!", new_block.get_hash());

                    // 从内存池中移除交易
                    for tx in &txs {
                        let txid_hex = HEXLOWER.encode(tx.get_id());
                        GLOBAL_MEMORY_POOL.remove(txid_hex.as_str());
                    }
                    // 广播新区块
                    let nodes = GLOBAL_NODES.get_nodes();
                    for node in &nodes {
                        if node_addr.eq(node.get_addr().as_str()) {
                            continue;
                        }
                        send_inv(
                            node.get_addr().as_str(),
                            OpType::Block,
                            &vec![new_block.get_hash_bytes()],
                        );
                    }
                }
            }
            Package::Version {
                addr_from,
                version,
                best_height,
            } => {
                info!("version = {}, best_height = {}", version, best_height);
                let local_best_height = blockchain.get_best_height();
                if local_best_height < best_height {
                    send_get_blocks(addr_from.as_str());
                }
                if local_best_height > best_height {
                    send_version(addr_from.as_str(), blockchain.get_best_height());
                }
                // 记录节点地址
                if GLOBAL_NODES.node_is_known(peer_addr.to_string().as_str()) == false {
                    GLOBAL_NODES.add_node(peer_addr.to_string());
                }
            }
        }
    }
    let _ = stream.shutdown(Shutdown::Both);
    Ok(())
}

/// 统一发送请求
fn send_data(addr: SocketAddr, pkg: Package) {
    info!("send package: {:?}", &pkg);
    let stream = TcpStream::connect(addr);
    if stream.is_err() {
        error!("The {} is not valid", addr);
        // 驱逐不健康的 Node
        GLOBAL_NODES.evict_node(addr.to_string().as_str());
        return;
    }
    let mut stream = stream.unwrap();
    let _ = stream.set_write_timeout(Option::from(Duration::from_millis(TCP_WRITE_TIMEOUT)));
    let _ = serde_json::to_writer(&stream, &pkg);
    let _ = stream.flush();
}

#[cfg(test)]
mod tests {
    use super::Server;
    use crate::Blockchain;

    #[test]
    fn test_new_server() {
        let blockchain = Blockchain::create_blockchain("1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa");
        let server = Server::new(blockchain);
        server.run("127.0.0.1:2001");
    }
}
