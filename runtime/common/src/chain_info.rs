use serde::{Deserialize, Serialize};

/// For now we only have one adapter type: Evm, later on we can add other non-evm networks
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ChainAdapterType {
    Evm,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StandardChainInfo {
    pub adapter:  ChainAdapterType,
    pub seda_id:  String,
    pub tag:      String,
    pub contract: String,
}
