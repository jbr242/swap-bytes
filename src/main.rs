use futures::stream::StreamExt;
use libp2p::{
    gossipsub, mdns, noise, swarm::NetworkBehaviour, swarm::SwarmEvent, tcp, yamux
};
use std::error::Error;
use std::time::Duration;
use tokio::{io, io::AsyncBufReadExt, select};

/// Defines the behaviour of the chat application, including mDNS and GossipSub protocols.
#[derive(NetworkBehaviour)]
struct ChatBehaviour {
    mdns: mdns::tokio::Behaviour,    // Multicast DNS (mDNS) for peer discovery
    gossipsub: gossipsub::Behaviour,  // GossipSub for pub/sub messaging
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
            })
        })?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(60)))  // Configure idle connection timeout
        .build();

    let topic = gossipsub::IdentTopic::new("chat"); // Define the chat topic
    swarm.behaviour_mut().gossipsub.subscribe(&topic)?; // Subscribe to the chat topic

    // Listen on specified TCP and UDP ports
    swarm.listen_on("/ip4/0.0.0.0/tcp/60489".parse()?)?;
    swarm.listen_on("/ip4/0.0.0.0/udp/60565/quic-v1".parse()?)?;

    let mut stdin = io::BufReader::new(io::stdin()).lines(); // Read lines from standard input
    println!("Enter chat messages one line at a time");

    loop {
        select! {
            // Handle new input from stdin
            Ok(Some(line)) = stdin.next_line() => {
                // Publish the message to the chat topic
                if let Err(err) = swarm.behaviour_mut().gossipsub.publish(topic.clone(), line.as_bytes()) {
                    println!("Error publishing: {:?}", err);
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
                    }
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::Mdns(mdns::Event::Expired(list))) => {
                    for (peer_id, _multiaddr) in list {
                        // Remove expired peers from GossipSub
                        swarm.behaviour_mut().gossipsub.remove_explicit_peer(&peer_id);
                    }
                }
                SwarmEvent::Behaviour(ChatBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                    propagation_source: peer_id,
                    message_id: id,
                    message,
                })) => {
                    // Print received messages
                    println!(
                        "Got message: '{}' with id: {id} from peer: {peer_id}",
                        String::from_utf8_lossy(&message.data),
                    );
                }
                _ => {}
            }
        }
    }
}
