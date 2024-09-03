use crate::utils::split_string;
use crate::behaviour::ChatBehaviour;
use libp2p::Multiaddr;
use libp2p::PeerId;
use libp2p::kad;
use libp2p::Swarm;
use std::collections::HashSet;
use std::error::Error;
use libp2p::gossipsub;

// pub fn dial_peer(swarm: &mut Swarm<ChatBehaviour>, peer_id: PeerId, address: Multiaddr) -> Result<(), Box<dyn Error>> {
//     println!("Dialing peer: {} at {}", peer_id, address);
//     swarm.dial(address.clone())?;
//     Ok(())
// }

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
        "/help" => {
            println!("Available commands:");
            println!("/help - Show this help message");
            println!("/peers - List all connected peers");
            println!("/nickname <nickname> - Set your nickname");
            println!("/id - Show your peer ID");
            println!("/join <topic> - Join a topic");
            println!("/topic - List currently subscribed topic");
            println!("/topics - List available topics");
        }
        "/peers" => {
            let peers = swarm.connected_peers();
            println!("Connected peers:");
            for peer in peers {
                println!("{}", peer);
            }
        }
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
        "/id" => {
            println!("Your peer ID is: {}", self_peer_id);
        }
        "/join" => {
            let gossipsub = &mut swarm.behaviour_mut().gossipsub;
            let current_topics: Vec<_> = gossipsub.topics().collect();
            //leave original topic first
            let topic = gossipsub::IdentTopic::new(current_topics[0].to_string());
            swarm.behaviour_mut().gossipsub.unsubscribe(&topic)?;
            let topic = gossipsub::IdentTopic::new(args.get(1).expect("No topic given"));
            swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
            println!("Joined topic: {}", topic);
        }
        "/topic" => {
            let topics = swarm.behaviour_mut().gossipsub.topics();
            println!("Currently subsribed topic:");
            for topic in topics {
                println!("{}", topic);
            }
        }
        "/topics" => {
            let allowed_topics: HashSet<&str> = ["chat", "movies", "books", "music"].iter().cloned().collect();
            println!("Available topics:");
            for topic in allowed_topics {
                println!("{}", topic);
            }
        }
        // "/send" => {
        //     if let (Some(filename), Some(peer_id_str), Some(address_str)) = (args.get(1), args.get(2), args.get(3)) {
        //         let peer_id = peer_id_str.parse::<PeerId>()?;
        //         let address = address_str.parse::<Multiaddr>()?;

        //         // Dial the peer to initiate the file transfer
        //         dial_peer(swarm, peer_id, address)?;

        //         // Implement the file sending logic here, such as using a custom protocol or streams
        //         println!("Preparing to send file: {} to peer: {}", filename, peer_id);
        //         // Example: send_file(swarm, peer_id, filename); // Function to handle the actual file transfer
        //     } else {
        //         println!("Usage: /send <filename> <peer_id> <multiaddr>");
        //     }
        // }

        _=> {
            println!("Unexpected command");
        }
    }
    Ok({})
}
