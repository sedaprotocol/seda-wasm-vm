use core::fmt;

use serde::{Deserialize, Serialize};

use super::HttpFetchAction;
use crate::events::Event;

// TODO: all action types with Vec<u8> can just be the Bytes type.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PromiseAction {
    CallSelf(CallSelfAction),
    DatabaseSet(DatabaseSetAction),
    DatabaseGet(DatabaseGetAction),
    Http(HttpFetchAction),
    ChainView(ChainViewAction),
    ChainSendTx(ChainSendTxAction),
    ChainTxStatus(ChainTxStatusAction),
    TriggerEvent(TriggerEventAction),
    P2PBroadcast(P2PBroadcastAction),
}

impl PromiseAction {
    #[cfg(not(target_family = "wasm"))]
    pub fn is_limited_action(&self) -> bool {
        !matches!(self, Self::CallSelf(_) | Self::Http(_))
    }
}

impl fmt::Display for PromiseAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CallSelf(_) => write!(f, "call_self"),
            Self::DatabaseSet(_) => write!(f, "db_set"),
            Self::DatabaseGet(_) => write!(f, "db_get"),
            Self::Http(_) => write!(f, "http"),
            Self::ChainView(_) => write!(f, "chain_view"),
            Self::ChainSendTx(_) => write!(f, "chain_send_tx"),
            Self::ChainTxStatus(_) => write!(f, "chain_tx_status"),
            Self::TriggerEvent(_) => write!(f, "trigger_event"),
            Self::P2PBroadcast(_) => write!(f, "p2p_broadcast"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CallSelfAction {
    pub function_name: String,
    pub args:          Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseSetAction {
    pub key:   String,
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseGetAction {
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChainViewAction {
    pub seda_id:     String,
    pub contract_id: String,
    pub method_name: String,
    pub args:        Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChainSendTxAction {
    pub seda_id:     String,
    pub contract_id: String,
    pub method_name: String,
    pub args:        Vec<u8>,
    pub deposit:     Option<u128>,
    pub gas:         Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChainTxStatusAction {
    pub seda_id: String,
    pub tx_id:   Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MainChainCallAction {
    pub identity_addr: String,
    pub contract:      String,
    pub msg:           Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MainChainViewAction {
    pub contract: String,
    pub query:    Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MainChainQueryAction {
    pub path_and_query: String,
    pub query:          Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TriggerEventAction {
    pub event:       Event,
    pub delay_in_ms: Option<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct P2PBroadcastAction {
    pub data: Vec<u8>,
}
