use crate::back_end::commands;
use crate::back_end::behaviour;
use crate::back_end::utils;

use futures::StreamExt;
use libp2p::{
    gossipsub, mdns, noise, swarm::SwarmEvent, tcp, yamux, kad, PeerId, 
};
use libp2p::kad::store::MemoryStore;
use libp2p::kad::Mode;
use std::error::Error;
use libp2p::kad::QueryId;
use std::time::Duration;
use std::collections::HashMap;
use tokio::{io, io::AsyncBufReadExt, select};

use behaviour::{ChatBehaviour, ChatBehaviourEvent};

pub async fn start_swarm_builder() -> Result<(), Box<dyn Error>> {
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
    let mut has_set_name = false;
    let mut pending_queries: HashMap<QueryId, (PeerId, String)> = HashMap::new();
    let self_peer_id = swarm.local_peer_id().clone();
    
    println!("Enter topic to subsribe to, Click enter to use default topic");
    let topics = stdin.next_line().await.unwrap().unwrap();
    if topics.is_empty() {
        let topic = gossipsub::IdentTopic::new("chat".to_string());
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    } else {
        let topic = gossipsub::IdentTopic::new(topics);
        swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
    }
    
    
    swarm.behaviour_mut().kademlia.set_mode(Some(Mode::Server));
    // Listen on specified TCP and UDP ports
    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/udp/0/quic-v1".parse()?)?;

    // Start the event handler
    println!("Enter chat messages one line at a time");
    loop {
        select! {
            Ok(Some(mut line)) = stdin.next_line() =>  {
                if line.starts_with("/") {
                    commands::handle_command(line, &mut swarm, self_peer_id)?;
                } else {
                    let current_topic: Vec<_> = swarm.behaviour_mut().gossipsub.topics().collect();
                    let topic = gossipsub::IdentTopic::new(current_topic[0].to_string());
                    line = format!("[{topic}]: {line}");
                    // Publish the message to the chat topic
                    if let Err(err) = swarm.behaviour_mut().gossipsub.publish(topic, line.as_bytes()) {
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
                        //if user has not set nickname
                        if !has_set_name {
                            let nickname_record = kad::Record {
                                key: kad::RecordKey::new(&self_peer_id.to_string()),
                                value: nickname.as_bytes().to_vec(),
                                publisher: None,
                                expires: None,
                            };
                            match swarm.behaviour_mut().kademlia.put_record(nickname_record, kad::Quorum::One) {
                                Ok(_) => {
                                    // If the record is stored successfully, set has_set_name to true
                                    has_set_name = true;
                                }
                                Err(e) => {
                                    eprintln!("Failed to store record: {:?}", e);
                                }
                            }
                        }
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
                    message, ..
                })) => {
                    {
                        if let Ok(msg) = String::from_utf8(message.data.clone()) {
                            // Start a query to get the nickname from the DHT
                            let query_id = swarm.behaviour_mut().kademlia.get_record(kad::RecordKey::new(&peer_id.to_string()));
                            //get topic of message

                            // Store the message and the peer ID with the query ID for later use
                            pending_queries.insert(query_id, (peer_id.clone(), msg));
                        }
                    }
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::Kademlia(kad::Event::OutboundQueryProgressed {id, result, ..})) => {
                    match result {
                        // Get record return result
                        kad::QueryResult::GetRecord(Ok(
                            kad::GetRecordOk::FoundRecord(kad::PeerRecord {
                                record: kad::Record { value, ..},
                                ..
                            })
                        )) => {
                            if let Some((peer_id, msg)) = pending_queries.remove(&id) {
                                match std::str::from_utf8(&value) {
                                    Ok(nickname) => {
                                        println!("{nickname}: {msg}");
                                    }
                                    Err(_) => {
                                        println!("Failed to decode nickname for peer {peer_id}, but received: {msg}");
                                    }
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
