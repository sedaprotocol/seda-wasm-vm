use lazy_static::lazy_static;
use seda_config::StandardChainInfo;

use super::shared_memory_get;
use crate::{FromBytes, Result};

pub const LATEST_PROXY_ADDRESS: &str = "latest_proxy_address";

lazy_static! {
    pub static ref IDENTITIES_ADDR: Vec<String> = get_identities_addr();
    pub static ref CONTRACT_ADDRESS: String = get_oracle_contract_id().expect("Could not decode oracle contract id");
    pub static ref CHAIN_INFOS: Vec<StandardChainInfo> = get_chain_infos().expect("Could not decode chain infos");
}

pub fn get_chain_infos() -> Result<Vec<StandardChainInfo>> {
    let chain_infos_encoded = std::env::var("CHAIN_INFOS").expect("Env 'CHAIN_INFOS' does not exist on the WASM VM");

    Ok(serde_json::from_str(&chain_infos_encoded)?)
}

pub fn get_oracle_contract_id() -> Result<String> {
    // This is for when the user has configured a contract address and want to use that instead of the fetched proxy
    // address
    if let Ok(contract_id) = std::env::var("ORACLE_CONTRACT_ID") {
        if !contract_id.is_empty() {
            return Ok(contract_id);
        }
    }

    String::from_bytes_vec(shared_memory_get(LATEST_PROXY_ADDRESS)?)
}

pub fn get_identities_addr() -> Vec<String> {
    let identities_addr_encoded =
        std::env::var("IDENTITIES_ADDR").expect("Env 'IDENTITIES_ADDR' does not exist on the WASM VM");

    serde_json::from_str(&identities_addr_encoded)
        .expect("Env 'IDENTITIES_ADDR' is not correctly encoded on the WASM VM")
}

pub fn get_local_bn254_public_key() -> String {
    std::env::var("BN254_PUBLIC_KEY").expect("Env 'BN254_PUBLIC_KEY' does not exist on the WASM VM")
}
