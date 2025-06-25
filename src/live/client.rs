/// Client implementation of the live protocol
use std::net::{TcpListener, TcpStream};

use tokio_tungstenite::tungstenite::{
    connect, http::Uri, stream::MaybeTlsStream, ClientRequestBuilder, WebSocket,
};

use super::{msg::Msg, session::Session};

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

    /// Just sending a Msg on the socket
    fn send_msg(&mut self, msg: &Msg) {
        self.socket.send(msg.into_ws_msg().unwrap());
    }

    /// Send a message and wait for a specific response in return
    fn send_msg_and_wait(&mut self, msg: &Msg) {
        self.socket.send(msg.into_ws_msg().unwrap());
        // TODO oups ?? how to do that ?
    }

    /// Get all available session for a given group id
    pub fn get_sessions(&mut self, group_id: String) -> Result<Vec<Session>, std::io::Error> {
        self.send_msg(&Msg::GetSessions { group_id });
        Ok(vec![])
    }
}
