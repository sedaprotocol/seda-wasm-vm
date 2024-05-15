use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct P2PMessage {
    pub source: Option<String>,
    pub data:   Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UnicastCommand {
    pub peer_id: String,
    pub data:    Vec<u8>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AddPeerCommand {
    pub multi_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RemovePeerCommand {
    pub peer_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum P2PCommand {
    Broadcast(Vec<u8>),
    Unicast(UnicastCommand),
    AddPeer(AddPeerCommand),
    RemovePeer(RemovePeerCommand),
    DiscoverPeers,
}
