use crate::{
    Block, BlockInTransit, Blockchain, MemoryPool, Nodes, Transaction, UTXOSet, GLOBAL_CONFIG,
};
use data_encoding::HEXLOWER;
use log::{debug, error};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Deserializer;
use std::error::Error;
use std::io::{BufReader, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

/// 版本硬编码
const NODE_VERSION: usize = 1;

/// 中心节点硬编码
const CENTER_NODE_ADDR: &str = "127.0.0.1:2001";

/// 节点地址
static GLOBAL_NODES: Lazy<Nodes> = Lazy::new(|| {
    let nodes = Nodes::new();
    // 记录中心地址
    nodes.add_node(String::from(CENTER_NODE_ADDR));
    return nodes;
});

/// 交易内存池
static GLOBAL_MEMORY_POOL: Lazy<MemoryPool> = Lazy::new(|| MemoryPool::new());

/// 传输中的Block, 用于来跟踪已下载的块, 这能够实现从不同的节点下载块
static GLOBAL_BLOCKS_IN_TRANSIT: Lazy<BlockInTransit> = Lazy::new(|| BlockInTransit::new());

/// 网络读写超时
const TCP_WRITE_TIMEOUT: u64 = 1000;

pub struct Server {
    blockchain: Blockchain,
}

impl Server {
    pub fn new(blockchain: Blockchain) -> Server {
        Server { blockchain }
    }

    pub fn run(&self, addr: SocketAddr) {
        let listener = TcpListener::bind(addr).unwrap();
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
enum OpType {
    Tx,
    Block,
}

#[derive(Debug, Serialize, Deserialize)]
enum Package {
    Addr {
        addr_list: Vec<String>,
    },
    Block {
        block: Vec<u8>,
    },
    GetBlocks,
    GetData {
        op_type: OpType,
        id: Vec<u8>,
    },
    Inv {
        op_type: OpType,
        items: Vec<Vec<u8>>,
    },
    Tx {
        transaction: Vec<u8>,
    },
    Version {
        version: usize,
        best_height: usize,
    },
}

fn serve(blockchain: Blockchain, stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let peer_addr = stream.peer_addr()?;
    let reader = BufReader::new(&stream);
    let pkg_reader = Deserializer::from_reader(reader).into_iter::<Package>();
    for pkg in pkg_reader {
        let pkg = pkg?;
        debug!("Receive request from {}: {:?}", peer_addr, pkg);
        match pkg {
            Package::Addr { addr_list } => {
                for addr in &addr_list {
                    GLOBAL_NODES.add_node(addr.clone());
                }
                debug!("There are {} known nodes now!", GLOBAL_NODES.len());

                for node in &GLOBAL_NODES.get_nodes() {
                    send_data(node.parse_socket_addr(), Package::GetBlocks);
                }
            }
            Package::Block { block } => {
                let block = Block::deserialize(block.as_slice());
                blockchain.add_block(&block);
                debug!("Added block {}", block.get_hash());

                if GLOBAL_BLOCKS_IN_TRANSIT.len() > 0 {
                    let block_hash = GLOBAL_BLOCKS_IN_TRANSIT.first().unwrap();
                    send_data(
                        peer_addr,
                        Package::GetData {
                            op_type: OpType::Block,
                            id: block_hash.clone(),
                        },
                    );
                    GLOBAL_BLOCKS_IN_TRANSIT.remove(block_hash.as_slice());
                } else {
                    let utxo_set = UTXOSet::new(blockchain.clone());
                    utxo_set.reindex();
                }
            }
            Package::GetBlocks => {
                let blocks = blockchain.get_block_hashes();
                send_data(
                    peer_addr,
                    Package::Inv {
                        op_type: OpType::Block,
                        items: blocks,
                    },
                );
            }
            Package::GetData { op_type, id } => match op_type {
                OpType::Block => {
                    if let Some(block) = blockchain.get_block(id.as_slice()) {
                        send_data(
                            peer_addr,
                            Package::Block {
                                block: block.serialize(),
                            },
                        )
                    }
                }
                OpType::Tx => {
                    let txid_hex = HEXLOWER.encode(id.as_slice());
                    if let Some(tx) = GLOBAL_MEMORY_POOL.get(txid_hex.as_str()) {
                        send_data(
                            peer_addr,
                            Package::Tx {
                                transaction: tx.serialize(),
                            },
                        );
                    }
                }
            },
            Package::Inv { op_type, items } => match op_type {
                OpType::Block => {
                    let block_hash = items.get(0).unwrap();
                    send_data(
                        peer_addr,
                        Package::GetData {
                            op_type: OpType::Block,
                            id: block_hash.to_vec(),
                        },
                    );
                    GLOBAL_BLOCKS_IN_TRANSIT.add_blocks(items[1..].as_ref());
                }
                OpType::Tx => {
                    let txid = items.get(0).unwrap();
                    let txid_hex = HEXLOWER.encode(txid);

                    // 检查交易池，不包含哈希则下载
                    if GLOBAL_MEMORY_POOL.containes(txid_hex.as_str()) {
                        send_data(
                            peer_addr,
                            Package::GetData {
                                op_type: OpType::Tx,
                                id: txid.clone(),
                            },
                        );
                    }
                }
            },
            Package::Tx { transaction } => {
                // 记录交易到内存池
                let tx = Transaction::deserialize(transaction.as_slice());
                let txid = tx.get_id_bytes();
                GLOBAL_MEMORY_POOL.add(tx);

                let node_addr = GLOBAL_CONFIG.get_addr();
                let center_node = GLOBAL_NODES.first().unwrap();
                // 中心节点
                if node_addr.eq(center_node.get_addr().as_str()) {
                    let nodes = GLOBAL_NODES.get_nodes();
                    for node in &nodes {
                        if node_addr.eq(node.get_addr().as_str()) {
                            continue;
                        }
                        if peer_addr.to_string().eq(node.get_addr().as_str()) {
                            continue;
                        }
                        send_data(
                            node.parse_socket_addr(),
                            Package::Inv {
                                op_type: OpType::Tx,
                                items: vec![txid.clone()],
                            },
                        );
                    }
                }
                // 矿工节点
                if GLOBAL_MEMORY_POOL.len() >= 2 && GLOBAL_CONFIG.is_miner() {
                    // 挖矿奖励
                    let coinbase_tx = Transaction::new_coinbase_tx(node_addr.as_str());
                    let mut txs = GLOBAL_MEMORY_POOL.get_all();
                    txs.push(coinbase_tx);

                    // 挖区块
                    let new_block = blockchain.mine_block(&txs);
                    let utxo_set = UTXOSet::new(blockchain.clone());
                    utxo_set.reindex();
                    debug!("New block {} is mined!", new_block.get_hash());

                    // 清理交易池
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
                        send_data(
                            node.parse_socket_addr(),
                            Package::Inv {
                                op_type: OpType::Block,
                                items: vec![new_block.get_hash_bytes()],
                            },
                        )
                    }
                }
            }
            Package::Version {
                version,
                best_height,
            } => {
                debug!("version = {}, best_height = {}", version, best_height);
                let local_best_height = blockchain.get_best_height();
                if local_best_height < best_height {
                    send_data(peer_addr, Package::GetBlocks);
                }
                if local_best_height > best_height {
                    send_data(
                        peer_addr,
                        Package::Version {
                            version: NODE_VERSION,
                            best_height: blockchain.get_best_height(),
                        },
                    )
                }
                // 记录节点地址
                if GLOBAL_NODES.node_is_known(peer_addr.to_string().as_str()) == false {
                    GLOBAL_NODES.add_node(peer_addr.to_string());
                }
            }
        }
    }
    Ok(())
}

/// 统一发送请求
fn send_data(addr: SocketAddr, pkg: Package) {
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
    use crate::server::{send_data, Package};
    use crate::Blockchain;

    #[test]
    fn test_new_server() {
        let blockchain = Blockchain::create_blockchain();
        let _ = crate::Server::new(blockchain);
    }

    #[test]
    fn test_send_data() {
        send_data("127.0.0.1:2001".parse().unwrap(), Package::GetBlocks);
    }
}
