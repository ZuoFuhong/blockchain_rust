use once_cell::sync::Lazy;

pub static GLOBAL_CONFIG: Lazy<Config> = Lazy::new(|| Config::new());

/// Node 配置
pub struct Config {
    ip: String,
    port: u16,
    miner: bool, // 矿工节点
}

impl Config {
    pub fn new() -> Config {
        Config {
            ip: String::from("127.0.0.1"),
            port: 3001,
            miner: false,
        }
    }

    pub fn get_addr(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn is_miner(&self) -> bool {
        self.miner
    }
}
