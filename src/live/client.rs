/// Client implementation of the live protocol
use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
};

use tokio_tungstenite::tungstenite::{
    connect, http::Uri, stream::MaybeTlsStream, ClientRequestBuilder, WebSocket,
};

use super::{
    msg::{ClientNum, ForwardedFile, ForwardedResult, Msg},
    session::Session,
};

#[derive(Eq, PartialEq)]
pub enum ClientRole {
    /// Default role, for anyone following a session
    Follower,
    /// When the client creates a session, it becames a leader client.
    /// When the session is stopped, it become a `Follower` again.
    Leader,
}

struct FollowerState {
    client_num: ClientNum,
    code: Option<ForwardedFile>,
    check_result: Option<ForwardedResult>,
}

pub struct LiveClient {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    /// The states of followers clients in the current session, only relevant for leader clients.
    /// It will be empty for follower clients.
    followers_states: HashMap<ClientNum, FollowerState>,
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

        let client = LiveClient {
            socket,
            followers_states: HashMap::new(),
        };

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
