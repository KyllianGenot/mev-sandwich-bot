use eyre::Result;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub rpc_url: String,
    pub rpc_ws_url: String,
    pub private_key: String,
    pub chain_id: u64,
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenvy::dotenv().ok();
        Ok(Config {
            rpc_url: std::env::var("RPC_URL")?,
            rpc_ws_url: std::env::var("RPC_WS_URL")?,
            private_key: std::env::var("PRIVATE_KEY")?,
            chain_id: std::env::var("CHAIN_ID").unwrap_or_else(|_| "1".to_string()).parse()?,
        })
    }
}