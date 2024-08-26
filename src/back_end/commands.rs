use crate::utils::split_string;
use crate::behaviour::ChatBehaviour;
use libp2p::PeerId;
use libp2p::kad;
use std::error::Error;

pub fn handle_command(
    line: String,
    swarm: &mut libp2p::Swarm<ChatBehaviour>,
    self_peer_id: PeerId,
) -> Result<(), Box<dyn Error>> {
    let args = split_string(&line);
    let kademlia = &mut swarm.behaviour_mut().kademlia;


    let cmd = if let Some(cmd) = args.get(0) {
        cmd 
    } else {
        println!("No command given");
        return Ok({});
    };

    match cmd.as_str() {
        "/nickname" =>{
            let local_peer_id_record = kad::RecordKey::new(&self_peer_id.to_string());
            let record = kad::Record {
                key: local_peer_id_record,
                value: args[1].as_bytes().to_vec(),
                publisher: None,
                expires: None,
            };
            kademlia
                .put_record(record, kad::Quorum::One)
                .expect("Failed to store record locally.");
        }
        // dial peerid
        // "/dial" => {
        //     if let Some(peer_id) = args.get(1) {
        //         let peer_id = PeerId::from_str(peer_id)?;
        //         let addr = kademlia.get_address(&peer_id)?;
        //         swarm.dial_addr(addr)?;
        //     } else {
        //         println!("No peer ID given");
        //     };
        // }
        _=> {
            println!("Unexpected command");
        }
    }
    Ok({})
}
