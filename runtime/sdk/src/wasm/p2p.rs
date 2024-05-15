use crate::P2PBroadcastAction;

// TODO: data could be cleaned up to a generic that implements our ToBytes trait
pub fn p2p_broadcast_message(data: Vec<u8>) {
    let p2p_broadcast_action = P2PBroadcastAction { data };
    let action = serde_json::to_string(&p2p_broadcast_action).unwrap();

    unsafe { super::raw::p2p_broadcast(action.as_ptr(), action.len() as u32) };
}
