use crate::utils::split_string;
use crate::behaviour::ChatBehaviour;
use libp2p::PeerId;
use libp2p::kad;
use std::collections::HashSet;
use std::error::Error;
use std::str::FromStr;
use libp2p::gossipsub;

use super::file_transfer::FileRequest;
use super::private_message::PrivateMessage;



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
            println!("/requestfile <peer_id> <filename> - Request a file from a peer");

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
        "/requestfile" => {
            //test if filename and peer id are provided
            if args.len() < 3 {
                println!("Please provide a peer ID and a filename");
                return Ok({});
            }
            let file_transfer = &mut swarm.behaviour_mut().file_transfer;
            let peer_id_str = &args[1];


            let peer_id = match PeerId::from_str(peer_id_str) {
                Ok(pid) => pid,
                Err(err) => {
                    eprintln!("Invalid Peer ID '{}': {}", peer_id_str, err);
                    return Ok(());
                }
            };
            let filename = &args[2];
            let request = FileRequest(filename.to_string());
            file_transfer.send_request(peer_id, request)?;
            println!("Sent file request for {} to {}", filename, peer_id);
        }

        "/msg" => {
            let private_message = &mut swarm.behaviour_mut().private_message;
            let peer_id_str = &args[1];
            let peer_id = match PeerId::from_str(peer_id_str) {
                Ok(pid) => pid,
                Err(err) => {
                    eprintln!("Invalid Peer ID '{}': {}", peer_id_str, err);
                    return Ok(());
                }
            };
            let self_peer_id_str = self_peer_id.to_string();
            let message = args[2..].join(" ");
            let priv_message = PrivateMessage {
                sender: self_peer_id_str,
                message,
            };
            match private_message.send_request(peer_id, priv_message) {
                Ok(_) => println!("Sent private message to {}", peer_id),
                Err(e) => eprintln!("Failed to send private message to {}: {:?}", peer_id, e),
            }
        }
        _=> {
            println!("Unexpected command");
        }
    }
    Ok({})
}
