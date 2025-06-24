/// Client implementation of the live protocol
use std::net::{TcpListener, TcpStream};

use tokio_tungstenite::tungstenite::{
    connect, http::Uri, stream::MaybeTlsStream, ClientRequestBuilder, WebSocket,
};

#[derive(Eq, PartialEq)]
pub enum ClientRole {
    /// Default role, for anyone following a session
    Follower,
    /// When the client creates a session, it becames a leader client
    Leader,
}

pub struct LiveClient {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
}

impl LiveClient {
    pub fn connect(
        domain: &str,
        port: u16,
        client_id: String,
    ) -> Result<LiveClient, std::io::Error> {
        let uri: Uri = format!("ws://{}:{}", domain, port).parse().unwrap(); // todo fix unwrap
        let builder = ClientRequestBuilder::new(uri);
        let (socket, _) = connect(builder).unwrap();

        let client = LiveClient { socket };

        Ok(client)
    }

    /// Get all available session for given group id
    pub fn get_sessions(group_id: String) -> Result<Vec<String>, std::io::Error> {
        Ok(vec![])
    }
}
