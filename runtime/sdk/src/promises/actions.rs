use core::fmt;

use serde::{Deserialize, Serialize};

use super::HttpFetchAction;
use crate::events::Event;

// TODO: all action types with Vec<u8> can just be the Bytes type.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub enum PromiseAction {
    CallSelf(CallSelfAction),
    DatabaseSet(DatabaseSetAction),
    DatabaseGet(DatabaseGetAction),
    Http(HttpFetchAction),
    ChainView(ChainViewAction),
    ChainSendTx(ChainSendTxAction),
    ChainTxStatus(ChainTxStatusAction),
    TriggerEvent(TriggerEventAction),
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
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct CallSelfAction {
    pub function_name: String,
    pub args:          Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct DatabaseSetAction {
    pub key:   String,
    pub value: Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct DatabaseGetAction {
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct ChainViewAction {
    pub seda_id:     String,
    pub contract_id: String,
    pub method_name: String,
    pub args:        Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct ChainSendTxAction {
    pub seda_id:     String,
    pub contract_id: String,
    pub method_name: String,
    pub args:        Vec<u8>,
    pub deposit:     Option<u128>,
    pub gas:         Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct ChainTxStatusAction {
    pub seda_id: String,
    pub tx_id:   Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct MainChainCallAction {
    pub identity_addr: String,
    pub contract:      String,
    pub msg:           Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct MainChainViewAction {
    pub contract: String,
    pub query:    Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct MainChainQueryAction {
    pub path_and_query: String,
    pub query:          Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[cfg_attr(test, derive(arbitrary::Arbitrary, PartialEq))]
pub struct TriggerEventAction {
    pub event:       Event,
    pub delay_in_ms: Option<u32>,
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::events::EventData;

    #[test]
    fn test_promise_action() {
        let action = PromiseAction::CallSelf(CallSelfAction {
            function_name: "test".to_string(),
            args:          vec!["arg1".to_string(), "arg2".to_string()],
        });
        assert_eq!(action.to_string(), "call_self");
        assert!(!action.is_limited_action());

        let action = PromiseAction::DatabaseSet(DatabaseSetAction {
            key:   "key".to_string(),
            value: vec![1, 2, 3],
        });
        assert_eq!(action.to_string(), "db_set");
        assert!(action.is_limited_action());

        let action = PromiseAction::DatabaseGet(DatabaseGetAction { key: "key".to_string() });
        assert_eq!(action.to_string(), "db_get");
        assert!(action.is_limited_action());

        let action = PromiseAction::Http(HttpFetchAction {
            url:     "url".to_string(),
            options: Default::default(),
        });
        assert_eq!(action.to_string(), "http");
        assert!(!action.is_limited_action());

        let action = PromiseAction::ChainView(ChainViewAction {
            seda_id:     "seda_id".to_string(),
            contract_id: "contract_id".to_string(),
            method_name: "method_name".to_string(),
            args:        vec![1, 2, 3],
        });
        assert_eq!(action.to_string(), "chain_view");
        assert!(action.is_limited_action());

        let action = PromiseAction::ChainSendTx(ChainSendTxAction {
            seda_id:     "seda_id".to_string(),
            contract_id: "contract_id".to_string(),
            method_name: "method_name".to_string(),
            args:        vec![1, 2, 3],
            deposit:     Some(100),
            gas:         Some(100),
        });
        assert_eq!(action.to_string(), "chain_send_tx");
        assert!(action.is_limited_action());

        let action = PromiseAction::ChainTxStatus(ChainTxStatusAction {
            seda_id: "seda_id".to_string(),
            tx_id:   vec![1, 2, 3],
        });
        assert_eq!(action.to_string(), "chain_tx_status");
        assert!(action.is_limited_action());

        let action = PromiseAction::TriggerEvent(TriggerEventAction {
            event:       Event::new("test", EventData::Vm(Default::default())),
            delay_in_ms: Some(100),
        });
        assert_eq!(action.to_string(), "trigger_event");
        assert!(action.is_limited_action());
    }
}
