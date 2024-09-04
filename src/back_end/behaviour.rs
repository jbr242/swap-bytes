use libp2p::{request_response, PeerId};
use libp2p::{
    gossipsub, kad, mdns, swarm::NetworkBehaviour,
};
use libp2p::kad::store::MemoryStore;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileRequest(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileResponse {
    pub filename: String, // To store the name of the file
    pub data: Vec<u8>,    // To store the actual file content
}

#[derive(NetworkBehaviour)]
pub struct FileTransferBehaviour {
    pub request_response: libp2p::request_response::cbor::Behaviour<FileRequest, FileResponse>
}
impl FileTransferBehaviour {
    pub fn send_request(
        &mut self,
        peer_id: PeerId,
        request: FileRequest,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Send a request to the peer using the `request_response` protocol
        self.request_response.send_request(&peer_id, request);
        Ok(())

        
    }
    pub async fn handle_request(
        &mut self,
        request: FileRequest,
        channel: request_response::ResponseChannel<FileResponse>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let filename = request.0.clone();
        println!("Received request for file: {}", filename);

        let file_bytes = self.select_file(filename).await?;
        let file_response = FileResponse {
            filename: request.0.clone(),
            data: file_bytes,
        };
        self.request_response.send_response(channel, file_response).unwrap();
        Ok(())
    }

    async fn select_file(&self, filename: String) -> Result<Vec<u8>, std::io::Error> {
        let mut file_bytes = Vec::new();
        //get files from upload folder 
        let path = std::path::Path::new("uploads").join(filename);
        println!("Reading file: {:?}", path);
        let mut file = tokio::fs::File::open(path).await?;
        file.read_to_end(&mut file_bytes).await?;
        Ok(file_bytes)
    }

}

#[derive(NetworkBehaviour)]
pub struct ChatBehaviour {
    pub mdns: mdns::tokio::Behaviour,
    pub gossipsub: gossipsub::Behaviour,
    pub kademlia: kad::Behaviour<MemoryStore>,
    pub request_response: FileTransferBehaviour,
}
