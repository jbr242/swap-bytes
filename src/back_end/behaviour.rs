
use libp2p::{
    gossipsub, kad, mdns, swarm::NetworkBehaviour,
};
use libp2p::kad::store::MemoryStore;

use super::file_transfer::FileTransferBehaviour;
use super::private_message::PrivateMessageBehaviour;


#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub file_transfer: FileTransferBehaviour,
    pub private_message: PrivateMessageBehaviour,
}
