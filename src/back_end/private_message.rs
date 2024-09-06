use serde::{Deserialize, Serialize};
use libp2p::{request_response, swarm::NetworkBehaviour, PeerId};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateMessage{
    pub message: String,
    pub sender: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateMessageResponse(pub String);

#[derive(NetworkBehaviour)]
pub struct PrivateMessageBehaviour {
    pub request_response: libp2p::request_response::cbor::Behaviour<PrivateMessage, PrivateMessageResponse>
}
impl PrivateMessageBehaviour {
    pub fn send_request(
        &mut self,
        peer_id: PeerId,
        request: PrivateMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Send a request to the peer using the `request_response` protocol
        self.request_response.send_request(&peer_id, request);
        Ok(())  
    }
    pub async fn handle_request(
        &mut self,
        channel: request_response::ResponseChannel<PrivateMessageResponse>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        
        let private_message_response = PrivateMessageResponse("Recipient recieved message".to_string());
        self.request_response.send_response(channel, private_message_response).unwrap();
        Ok(())
    }
}