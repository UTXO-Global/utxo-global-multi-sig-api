use crate::config;
use ckb_sdk::{rpc::CkbRpcClient, NetworkType};

pub fn get_ckb_client() -> CkbRpcClient {
    let rpc_url: String = config::get("ckb_rpc");
    CkbRpcClient::new(rpc_url.as_str())
}

pub fn get_ckb_network() -> NetworkType {
    let network: String = config::get("network");
    match network.as_str() {
        "mainnet" => NetworkType::Mainnet,
        _ => NetworkType::Testnet,
    }
}
