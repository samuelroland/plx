/// Client implementation of the live protocol
use std::{collections::HashMap, fmt::Display, sync::mpsc::Sender, thread, time::Duration};

use tokio_tungstenite::tungstenite;

use super::{
    client_splitter::{ClientSplitter, SplitterInterface},
    msg::{
        Action, ClientNum, Event, ExoCheckResult, ForwardedFile, ForwardedResult,
        LiveProtocolError, SessionStats,
    },
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
    code: Option<HashMap<String, ForwardedFile>>,
    check_result: Option<ForwardedResult>,
}

struct SessionDetails {
    session: Session,
    stats: SessionStats,
}

pub struct LiveClient {
    mp: SplitterInterface,
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
        let (mut splitter, mp) = ClientSplitter::new();
        let domain = domain.to_string();
        thread::spawn(move || {
            splitter.start(&domain, port, client_id);
        });
        thread::sleep(Duration::from_secs(1));
        let client = LiveClient {
            mp,
            followers_states: HashMap::new(),
            session: None,
        };

        Ok(client)
    }

    pub fn disconnect(self) {
        // just do nothing, let the struct and self.mp drop to close the self.mp.send
        // on the other end the receiver.recv() will return none which should stop the Splitter
        // and at the same time close the websocket on drop
    }

    /// Just sending a Msg on the socket
    fn send_msg(&mut self, action: Action) {
        println!("Sending: {action:?}");
        let _ = self.mp.send.send(action).unwrap();
    }

    pub fn training_events_subscribe(&mut self, tx: Sender<Event>) {
        while let Some(a) = self.mp.training_recv.blocking_recv() {
            // emit tauri event
            println!("{a:?}");
            tx.send(a);
        }
    }

    /// Create a new session
    pub fn start_session(&mut self, name: &str, group_id: &str) -> Result<Session, String> {
        self.send_msg(Action::StartSession {
            name: name.to_string(),
            group_id: group_id.to_string(),
        });
        let event = self.mp.session_recv.blocking_recv();
        if let Some(Event::SessionStarted) = event {
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
        let event = self.mp.session_recv.blocking_recv();
        if let Some(Event::SessionStopped) = event {
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
        if let Some(Event::SessionJoined) = self.mp.session_recv.blocking_recv() {
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
    pub fn send_exo_switch(&mut self, path: String) {
        self.send_msg(Action::ExoSwitch { path });

        // println!("blocking_recv");
        // let event = self.mp.training_recv.blocking_recv();
        // println!("event = {event:?}");
        // if let Some(Event::ExoSwitched { .. }) = event {
        //     return Ok(());
        // }
        // if let Some(Event::Error(e)) = event {
        //     return Err(ProtocolError::Live(e));
        // }
        // Err(ProtocolError::UnexpectedMsg(format!(
        //     "Invalid message {:?} received after ExoSwitch",
        //     event
        // )))
    }

    /// Get all available session for a given group id
    pub fn get_sessions(&mut self, group_id: String) -> Result<Vec<Session>, ()> {
        self.send_msg(Action::GetSessions { group_id });
        let e = self.mp.session_recv.blocking_recv();
        if let Some(Event::SessionsList(list)) = e {
            return Ok(list);
        }
        Err(())
    }
}
