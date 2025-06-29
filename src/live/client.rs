/// Client implementation of the live protocol
use std::{collections::HashMap, fmt::Display, net::TcpStream};

use tokio_tungstenite::tungstenite::{
    self, connect, http::Uri, stream::MaybeTlsStream, ClientRequestBuilder, Message, WebSocket,
};

use super::{
    msg::{
        Action, ClientNum, Event, ExoCheckResult, ForwardedFile, ForwardedResult,
        LiveProtocolError, SessionStats,
    },
    server::{HEADER_LIVE_CLIENT_ID, HEADER_LIVE_PROTOCOL_VERSION, PROTOCOL_VERSION},
    session::Session,
};

#[derive(Eq, PartialEq, Debug, Clone)]
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

struct SessionDetails {
    session: Session,
    stats: SessionStats,
}

pub struct LiveClient {
    socket: WebSocket<MaybeTlsStream<TcpStream>>,
    /// The states of followers clients in the current session, only relevant for leader clients.
    /// It will be empty for follower clients.
    followers_states: HashMap<ClientNum, FollowerState>,

    /// When connected to a session, retain a few details locally
    session: Option<SessionDetails>,
}

#[derive(Debug)]
pub enum ProtocolError {
    Live(LiveProtocolError),
    Network(Box<tungstenite::Error>),
    UnexpectedMsg(String),
}

impl Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self {
                ProtocolError::Live(live_protocol_error) => {
                    format!("Live protocol error: {live_protocol_error}")
                }
                ProtocolError::Network(error) => format!("Network error: {error}"),
                ProtocolError::UnexpectedMsg(text) => {
                    format!("Unexpected message received from the server: {text}")
                }
            }
            .as_ref(),
        )
    }
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
            session: None,
        };

        Ok(client)
    }

    pub fn disconnect(mut self) {
        let _ = self.socket.close(None);
    }

    /// Just sending a Msg on the socket
    fn send_msg(&mut self, msg: Action) {
        println!("Sending: {msg:?}");
        let _ = self.socket.send(Message::Text(msg.try_into().unwrap()));
    }

    /// Receive an event, save Event::Stats but skip it and wait for next event
    fn receive_event(&mut self) -> Result<Event, String> {
        let received = Event::try_from(
            self.socket
                .read()
                .map_err(|e| e.to_string())?
                .into_text()
                .map_err(|_| "Received invalid message".to_string())?,
        )
        .map_err(|e| e.to_string());
        println!("Received {:?}", received);
        match &received {
            Ok(Event::Stats(stats)) => {
                if let Some(session_details) = &mut self.session {
                    session_details.stats = stats.clone();
                }
                self.receive_event() // wait for another event
            }
            _ => received,
        }
    }

    /// Create a new session
    pub fn start_session(&mut self, name: &str, group_id: &str) -> Result<Session, String> {
        self.send_msg(Action::StartSession {
            name: name.to_string(),
            group_id: group_id.to_string(),
        });
        let event = self.receive_event();
        if let Ok(Event::SessionStarted) = event {
            Ok(Session {
                name: name.to_string(),
                group_id: group_id.to_string(),
            })
        } else {
            Err(format!("{:?}", event))
        }
    }

    /// Create a new session
    pub fn stop_session(&mut self) -> Result<(), String> {
        self.send_msg(Action::StopSession);
        let event = self.receive_event();
        if let Ok(Event::SessionStopped) = event {
            Ok(())
        } else {
            Err(format!("{:?}", event))
        }
    }

    /// Join a session
    pub fn join_session(&mut self, name: &str, group_id: &str) -> Result<Session, String> {
        self.send_msg(Action::JoinSession {
            name: name.to_string(),
            group_id: group_id.to_string(),
        });
        let event = self.receive_event();
        if let Ok(Event::SessionJoined) = event {
            println!("Joined session '{name}'");
        }
        Ok(Session {
            name: name.to_string(),
            group_id: group_id.to_string(),
        })
    }

    /// Send a file content after a change
    pub fn send_file(&mut self, file: String, content: String) {
        self.send_msg(Action::SendFile { file, content });
    }

    /// Send a check result
    pub fn send_result(&mut self, check_result: ExoCheckResult) {
        self.send_msg(Action::SendResult { check_result });
    }

    /// Send a check result
    pub fn send_exo_switch(&mut self, path: String) -> Result<(), ProtocolError> {
        self.send_msg(Action::ExoSwitch { path });
        let event = self.receive_event();
        if let Ok(Event::ExoSwitched { .. }) = event {
            return Ok(());
        }
        if let Ok(Event::Error(e)) = event {
            return Err(ProtocolError::Live(e));
        }
        Err(ProtocolError::UnexpectedMsg(format!(
            "Invalid message {:?} received after ExoSwitch",
            event
        )))
    }

    /// Get all available session for a given group id
    pub fn get_sessions(&mut self, group_id: String) -> Result<Vec<Session>, std::io::Error> {
        self.send_msg(Action::GetSessions { group_id });
        if let Ok(msg) = self.socket.read() {
            if let Ok(Event::SessionsList(list)) = Event::try_from(msg.into_text().unwrap()) {
                return Ok(list);
            }
        }
        Ok(vec![])
    }
}
