use futures::stream::StreamExt;
use libp2p::bytes::Bytes;
use regex::Regex;
use libp2p::{
    gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux, 
};
use libp2p::kad;
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;
use std::error::Error;
use std::time::Duration;
use std::collections::HashMap;
use tokio::{io, io::AsyncBufReadExt, select};

/// Defines the behaviour of the chat application, including mDNS and GossipSub protocols.
#[derive(NetworkBehaviour)]
struct ChatBehaviour {
    mdns: mdns::tokio::Behaviour,    // Multicast DNS (mDNS) for peer discovery
    gossipsub: gossipsub::Behaviour,  // GossipSub for pub/sub messaging
    kademlia: kad::Behaviour<MemoryStore>
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    
    // Build and configure the libp2p swarm
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),  // Default TCP configuration
            noise::Config::new,      // Noise protocol configuration for encryption
            yamux::Config::default, // Yamux configuration for multiplexing
        )?
        .with_quic()  // Add QUIC support
        .with_behaviour(|key| {
            // Create a new instance of ChatBehaviour with mDNS and GossipSub
            Ok(ChatBehaviour {
                mdns: mdns::tokio::Behaviour::new(
                    mdns::Config::default(),      // Default mDNS configuration
                    key.public().to_peer_id(),    // Local peer ID
                )?,
                gossipsub: gossipsub::Behaviour::new(
                    gossipsub::MessageAuthenticity::Signed(key.clone()), // Signed message authenticity
                    gossipsub::Config::default(),  // Default GossipSub configuration
                )?,
                kademlia: kad::Behaviour::new(
                    key.public().to_peer_id(),
                    MemoryStore::new(key.public().to_peer_id()),
                ),
            })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))  // Configure idle connection timeout
        .build();

    //Let user select nickname
    let mut stdin = io::BufReader::new(io::stdin()).lines();
    println!("Enter your nickname");
    let nickname = stdin.next_line().await.unwrap().unwrap();
    
    let topic = gossipsub::IdentTopic::new("chat"); // Define the chat topic
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?; // Subscribe to the chat topic


    // Listen on specified TCP and UDP ports
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;
    

    let mut stdin = io::BufReader::new(io::stdin()).lines(); // Read lines from standard input
    println!("Enter chat messages one line at a time");

    loop {
        select! {
            Ok(Some(line)) = stdin.next_line() =>  {
                //if line starts with / then it is a command
                if line.starts_with("/") {
                    handle_command(line, &mut swarm.behaviour_mut().kademlia)?;
                } else {
                    // Publish the message to the chat topic
                    if let Err(err) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), line.as_bytes()) {
                        println!("Error publishing: {:?}", err);
                    }
                }
            }
            // Handle events from the swarm
            event = swarm.select_next_some() => match event {
                SwarmEvent::NewListenAddr { address, ..} => {
                    println!("Your node is listening on {address}");
                    
                
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns::Event::Discovered(list))) => {
                    for (peer_id, multiaddr) in list {
                        println!("mDNS discovered peer: {peer_id}, listening on {multiaddr}");
                        // Add discovered peers to GossipSub
                        swarm.behaviour_mut().gossipsub.add_explicit_peer(&peer_id);
                        swarm.behaviour_mut().kademlia.add_address(&peer_id, multiaddr);
                        let record = kad::Record {
                            key: kad::RecordKey::new(&swarm.local_peer_id().to_string()),
                            value: nickname.as_bytes().to_vec(),
                            publisher: None,
                            expires: None,
                        };
                        swarm.behaviour_mut().kademlia
                            .put_record(record, kad::Quorum::One)
                            .expect("Failed to store record locally.");
                    }
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, multiaddr) in list {
                        // Remove expired peers from GossipSub
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                        swarm.behaviour_mut().kademlia.remove_address(&peer_id, &multiaddr);
                    }
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => {
                    if let Ok(msg) = std::str::from_utf8(&message.data) {
                        println!("Received message from {peer_id}: {msg}");
                    }
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {result, ..})) => {
                    match result {
                        kad::QueryResult::GetRecord(Ok(
                            kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                                record: kad::Record { key, value, ..},
                                ..
                            })
                        )) => {
                            match std::str::from_utf8(&value) {
                                Ok(nickname) => {
                                    println!("Peer {nickname} is listening on ");
                                }
                                Err(_) => {
                                    println!("Peer what");
                                }
                            }
                        }

                        kad::QueryResult::GetRecord(Ok(_)) => {}
                        kad::QueryResult::GetRecord(Err(err)) => {
                            println!("Failed to get record {err:?}");
                        }
                        kad::QueryResult::PutRecord(Ok(kad::PutRecordOk {key })) => {
                            println!("Successfully put record {:?}", std::str::from_utf8(key.as_ref()).unwrap());
                        }
                        kad::QueryResult::PutRecord(Err(err)) => {
                            println!("Failed to put record {err:?}");
                        }
                        _ => {}
                    }
                }
                
                _ => {}
            }
        }
    }
}

fn handle_command(
    line: String,
    kademlia: &mut kad::Behaviour<MemoryStore>,
) -> Result<(), Box<dyn Error>> {
    let args = split_string(&line);

    let cmd = if let Some(cmd) = args.get(0) {
        cmd 
    } else {
        println!("No command given");
        return Ok({});
    };

    match cmd.as_str() {
        "/nickname" =>{
            // /nickname Josh "learning rust "ben adams" mystery 100
            let record = kad::Record {
                key: kad::RecordKey::new(&args[1]),
                value: args[1].as_bytes().to_vec(),
                publisher: None,
                expires: None,
            };
            kademlia
                .put_record(record, kad::Quorum::One)
                .expect("Failed to store record locally.");
        }
        _=> {
            println!("Unexpected command");
        }
    }
    Ok({})
}

fn split_string(input: &str) -> Vec<String> {
    let re = Regex::new(r#""([^"]*)"|\S+"#).unwrap();
    re.captures_iter(input)
        .map(|cap| cap.get(0).unwrap().as_str().to_string())
        .collect()
}