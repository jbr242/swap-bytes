# Swap Bytes P2P File Bartering Application
This is a peer-to-peer (P2P) file bartering application built using libp2p. The application allows users to chat in various topics, request files from peers, and exchange files over a decentralized network.

## Features
* Decentralized chat with customizable chatrooms.
* File exchange between peers.
* Peer discovery using mDNS and Kademlia.
* Encrypted communication with Noise protocol.
* Supports multiple topics for chat, including private messaging.
### Getting Started
#### Prerequisites
Ensure you have the following installed on your system:
* Rust (recommended: stable version)
* Tokio (async runtime)
#### Installation
Clone the repository:

```bash
git clone https://github.com/jbr242/swap-bytes.git
cd swap-bytes
```
### Running the Application
#### Compile the project:

``` bash
cargo build
```
#### Run the application:

```bash
cargo run
```
#### Enter your nickname:

Upon starting the application, you will be prompted to enter your nickname. This nickname will be used in the chat and stored in the DHT for other peers to discover. If you do not choose a nickname, your peerid will be set as your nickname

#### Select a chat topic:

You can choose from allowed topics like chat, movies, books, or music. Simply type the topic name or press Enter to join the default topic (chat).

### Bootstrapping the Network
The network bootstraps automatically via mDNS and Kademlia:

* mDNS: Discovers peers on the local network.
* Kademlia: Used for finding peers and storing/retrieving nicknames.
The application listens on random TCP and QUIC ports, which are printed upon startup.

### Commands
During the application runtime, you can use the following commands:

* /help: Show a help message.
* /peers: List all connected peers.
* /nickname <nickname>: Set your nickname.
* /id: Show your peer ID.
* /join <topic>: Join a new topic. You will automatically leave the current topic.
* /topic: Show the currently subscribed topic.
* /topics: List available topics.
* /requestfile <peer_id> <file_name> : Request a file from a peer.
* /msg <peer_id> <message> : Send a private message to a peer
* /exit : Exit program
### Examples
1. Sending a message:
   * Simply type your message and press Enter to send it to the current chat topic.
2. Joining a new topic:
  * Use the command /join <topic> to switch to a new topic (e.g., /join movies).
3. Requesting a file:
  * Use /requestfile <peer_id> <file_name> to request a file from another peer. Make sure the peer ID is valid and the file exists.
4. Setting your nickname:
  * Use /nickname <new_nickname> to update your nickname in the network.
5. Sending a private message:
  * Use /msg <peer_id> <message> to send a private message to a peer, useful for discussion of file trading!
### File Handling
* Uploads: Place files you want to share in the uploads folder. Files not found in this directory cannot be shared.
* Downloads: Received files are saved in the downloads folder with sanitized filenames to prevent directory traversal attacks.
