use std::{
    fmt::Display,
    time::{Instant, SystemTime},
};

use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};

/// The protocol defines the possible messages, sent from the clients or the server
/// Some messages can be used for a request of something, some others are responding to another request
/// We avoid naming it `Message` because there is already `tungstenite::Message`
#[derive(Serialize, Deserialize, Clone)]
pub enum Msg {
    StartSession {
        name: String,
        group_id: String,
    },
    StopSession {},    // the client_id will be used to verify the permission
    SessionStopped {}, // event to broadcast to all clients in the session
    GetSessions {
        group_id: String,
    },
    JoinSession {
        name: String,
        group_id: String,
    },
    SessionJoined {}, // as a confirmation that JoinSession worked
    LeaveSession {},
    // Stats for leaders about how much followers have joined
    Stats {
        followers_count: u16,
        leaders_count: u16,
    },

    // Code exo syncing
    SendFile {
        file: String,
        content: String,
    },
    ForwardFile(ClientNum, ForwardedFile),
    SendResult {
        check_id: u32,
        passed: bool,
    },
    ForwardResult(ClientNum, ForwardedResult),

    Error(LiveProtocolError),
}

/// Implement serialisation and deserialisation strategy for this Msg
impl Msg {
    pub fn from_ws_msg(ws_msg: &Message) -> Result<Msg, String> {
        match ws_msg {
            Message::Text(utf8) => serde_json::from_str::<Msg>(utf8)
                .map_err(|e| format!("Couldn't parse message: {e}")),
            _ => Err("Message was not in Text format".to_string()),
        }
    }
    pub fn into_ws_msg(&self) -> Result<Message, String> {
        let str = serde_json::to_string(&self).map_err(|e| e.to_string())?;
        let bytes = Utf8Bytes::from(str);
        Ok(Message::Text(bytes))
    }
}

/// An error sent from the server to clients after any message
/// that resolved in an error that is worth sending back to the client
#[derive(Serialize, Deserialize, Clone)]
pub enum LiveProtocolError {
    SessionNotFound,
    CannotJoinOtherSession,
}

impl Display for LiveProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            LiveProtocolError::SessionNotFound => "The session wasn't found.",
            LiveProtocolError::CannotJoinOtherSession => {
                "You cannot join another session without having left your current session."
            }
        };
        f.write_str(text)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ForwardedFile {
    /// The relative path inside the exo folder, like "main.cpp", "src/main.rs", "lib/image.h"
    file: String,
    content: String,
    /// The time where this code was received on the server
    time: std::time::SystemTime,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ForwardedResult {
    /// The relative path inside the exo folder, like "main.cpp", "src/main.rs", "lib/image.h"
    pub check_id: u32,
    pub passed: bool,
    /// The time where this result was received on the server
    pub time: std::time::SystemTime,
}

/// A incremental number attributed by the server to each client after session join, to let clients identify other clients.
/// This MUST NOT be derived from the secret client_id, this ClientNum is not secret but should be different at each session.
#[derive(Serialize, Deserialize, Clone)]
pub struct ClientNum(pub u16);
