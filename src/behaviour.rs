use libp2p::{
    gossipsub, kad, mdns, swarm::NetworkBehaviour,
};
use libp2p::kad::store::MemoryStore;

#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
}
