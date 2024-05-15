use seda_config::SedaId;

use crate::{
    ChainSendTxAction,
    ChainTxStatusAction,
    ChainViewAction,
    MainChainCallAction,
    MainChainQueryAction,
    MainChainViewAction,
    PromiseStatus,
};

pub fn chain_view<C: ToString, M: ToString>(
    seda_id: SedaId,
    contract_id: C,
    method_name: M,
    args: Vec<u8>,
) -> PromiseStatus {
    let chain_view_action = ChainViewAction {
        seda_id,
        contract_id: contract_id.to_string(),
        method_name: method_name.to_string(),
        args,
    };

    let action = serde_json::to_string(&chain_view_action).unwrap();
    let result_length = unsafe { super::raw::chain_view(action.as_ptr(), action.len() as u32) };
    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_length);
    }

    serde_json::from_slice(&result_data_ptr).expect("Could not deserialize chain_view")
}

pub fn chain_send_tx<C: ToString, M: ToString>(
    seda_id: SedaId,
    contract_id: C,
    method_name: M,
    args: Vec<u8>,
    gas: Option<u64>,
    deposit: Option<u128>,
) -> PromiseStatus {
    let chain_send_tx_action = ChainSendTxAction {
        seda_id,
        contract_id: contract_id.to_string(),
        method_name: method_name.to_string(),
        args,
        gas,
        deposit,
    };

    let action = serde_json::to_string(&chain_send_tx_action).unwrap();
    let result_length = unsafe { super::raw::chain_send_tx(action.as_ptr(), action.len() as u32) };
    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_length);
    }

    serde_json::from_slice(&result_data_ptr).expect("Could not deserialize chain_send_tx")
}

pub fn chain_tx_status(seda_id: SedaId, tx_id: Vec<u8>) -> PromiseStatus {
    let chain_tx_status_action = ChainTxStatusAction { seda_id, tx_id };

    let action = serde_json::to_string(&chain_tx_status_action).unwrap();
    let result_length = unsafe { super::raw::chain_tx_status(action.as_ptr(), action.len() as u32) };
    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_length);
    }

    serde_json::from_slice(&result_data_ptr).expect("Could not deserialize chain_tx_status")
}

pub fn main_chain_view<C: ToString>(contract_id: C, query: Vec<u8>) -> PromiseStatus {
    let chain_view_action = MainChainViewAction {
        contract: contract_id.to_string(),
        query,
    };

    let action = serde_json::to_vec(&chain_view_action).expect("Invalid main chain view action");
    let result_length = unsafe { super::raw::main_chain_view(action.as_ptr(), action.len() as u32) };
    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_length);
    }

    serde_json::from_slice(&result_data_ptr).expect("Could not deserialize main_chain_view")
}

pub fn main_chain_query<PP: ToString>(query: Vec<u8>, path_and_prefix: PP) -> PromiseStatus {
    let chain_query_action = MainChainQueryAction {
        query,
        path_and_query: path_and_prefix.to_string(),
    };

    let action = serde_json::to_vec(&chain_query_action).expect("Invalid main chain view action");
    let result_length = unsafe { super::raw::main_chain_query(action.as_ptr(), action.len() as u32) };
    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_length);
    }

    serde_json::from_slice(&result_data_ptr).expect("Could not deserialize main_chain_query")
}

pub fn main_chain_call<C: ToString, IA: ToString>(identity_addr: IA, contract_id: C, args: Vec<u8>) -> PromiseStatus {
    let chain_call_action = MainChainCallAction {
        contract:      contract_id.to_string(),
        msg:           args,
        identity_addr: identity_addr.to_string(),
    };

    let action = serde_json::to_vec(&chain_call_action).expect("Invalid main chain call action");
    let result_length = unsafe { super::raw::main_chain_call(action.as_ptr(), action.len() as u32) };
    let mut result_data_ptr = vec![0; result_length as usize];

    unsafe {
        super::raw::call_result_write(result_data_ptr.as_mut_ptr(), result_length);
    }

    serde_json::from_slice(&result_data_ptr).expect("Could not deserialize main_chain_call")
}
