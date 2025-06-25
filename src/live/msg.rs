use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};

/// The protocol defines the messages sent as generic message, it could be a request from the
/// client or the server or a response to another request
/// We avoid naming it `Message` because the websocket message type already use this name
#[derive(Serialize, Deserialize)]
pub enum Msg {
    CreateSession { name: String, group_id: String },
    DeleteSession {},  // the client_id will be used to verify the permission
    SessionStopped {}, // event to broadcast to all clients in the session
    GetSessions { group_id: String },
    JoinSession { name: String, group_id: String },
    SessionJoined {}, // as a confirmation that JoinSession worked
    // Stats for leaders about how much followers have joined
    Stats { followers: u32 },
    SendCode { file: String, content: String },
    SendResult { check_id: u32, passed: bool },

    Error(ProtocolError),
}

impl Msg {
    pub fn from_ws_msg(ws_msg: &Message) -> Result<Msg, String> {
        match ws_msg {
            Message::Text(utf8) => serde_json::from_str::<Msg>(utf8)
                .map_err(|e| format!("Couldn't parse message: {e}")),
            _ => Err("Message was not in Text format".to_string()),
        }
    }
    pub fn into_ws_msg(self) -> Result<Message, String> {
        let str = serde_json::to_string(&self).map_err(|e| e.to_string())?;
        let bytes = Utf8Bytes::from(str);
        Ok(Message::Text(bytes))
    }
}

/// An error sent from the server to clients after any message
/// that resolved in an error that is worth sending back to the client
#[derive(Serialize, Deserialize)]
enum ProtocolError {
    SessionNotFound,
}

impl Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            ProtocolError::SessionNotFound => "The session wasn't found",
        };
        f.write_str(text)
    }
}
