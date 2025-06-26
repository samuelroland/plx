/// Client implementation of the live protocol
use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
};

use tokio_tungstenite::tungstenite::{
    connect, http::Uri, stream::MaybeTlsStream, ClientRequestBuilder, Message, WebSocket,
};

use super::{
    msg::{Action, ClientNum, Event, ForwardedFile, ForwardedResult},
    server::{HEADER_LIVE_CLIENT_ID, HEADER_LIVE_PROTOCOL_VERSION, PROTOCOL_VERSION},
    session::Session,
};

#[derive(Eq, PartialEq, Debug)]
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
        let builder = ClientRequestBuilder::new(uri)
            .with_header(HEADER_LIVE_PROTOCOL_VERSION, PROTOCOL_VERSION)
            .with_header(HEADER_LIVE_CLIENT_ID, client_id);
        let (socket, _) = connect(builder).unwrap();

        let client = LiveClient {
            socket,
            followers_states: HashMap::new(),
        };

        Ok(client)
    }

    pub fn disconnect(mut self) {
        let _ = self.socket.close(None);
    }

    /// Just sending a Msg on the socket
    fn send_msg(&mut self, msg: Action) {
        let _ = self.socket.send(Message::Text(msg.try_into().unwrap()));
    }

    /// Create a new session
    pub fn start_session(&mut self, name: &str, group_id: String) -> Result<Session, String> {
        self.send_msg(Action::StartSession {
            name: name.to_string(),
            group_id: group_id.clone(),
        });
        let event = Event::try_from(self.socket.read().unwrap().into_text().unwrap()).unwrap();
        if let Event::SessionStarted = event {
            println!("session started !");
            Ok(Session {
                name: name.to_string(),
                group_id,
            })
        } else {
            Err(format!("{:?}", event))
        }
    }

    /// Get all available session for a given group id
    pub fn get_sessions(&mut self, group_id: String) -> Result<Vec<Session>, std::io::Error> {
        self.send_msg(Action::GetSessions { group_id });
        Ok(vec![])
    }
}
