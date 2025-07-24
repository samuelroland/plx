use chrono::serde::ts_seconds;
use chrono::{DateTime, Utc};
use specta_macros::Type;
use strum_macros::AsRefStr;
use typeshare::typeshare;

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Utf8Bytes;

// PROTOCOL CONCEPTS

/// A live session, this is the representation sent to clients
/// when listing all sessions or after session creation
#[derive(Serialize, Deserialize, Eq, Ord, PartialOrd, PartialEq, Clone, Debug, Type)]
#[typeshare]
pub struct Session {
    /// An arbitrary name defined by the leader to help followers choose the correct sessions
    /// among the multiple live sessions at the same time on the same group_id
    /// We imagine it could be named like "Course name - Teacher fullname"
    pub name: String,
    /// The group id is a way to group related sessions together.
    /// This can be an arbitrary string chosen by leader clients when creating a session.
    /// Listing available sessions can only be done via this group_id to filter the list
    /// By default, PLX clients will send the Git HTTPS link
    pub group_id: String,
}

/// A incremental number attributed by the server to each client after session join, to let clients identify other clients.
/// This MUST NOT be derived from the secret client_id, this ClientNum is not secret but should be different at each session.
#[derive(Serialize, Deserialize, Eq, PartialEq, Hash, Clone, Debug, Type)]
#[typeshare]
pub struct ClientNum(pub u16);

#[derive(Eq, PartialEq, Debug, Clone)]
#[typeshare]
/// The role of a client attributed when has joined a session
pub enum ClientRole {
    /// Default role, for anyone following a session
    Follower,
    /// When the client creates a session, it becames a leader client.
    /// When the session is stopped, it become a `Follower` again.
    Leader,
}

// MESSAGES

// TODO: Temporary copy in waiting of refactor to access that
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[serde(tag = "type", content = "content")]
#[typeshare]
pub enum CheckStatus {
    Passed,
    CheckFailed(String),
    BuildFailed(String),
    RunFailed(String),
}
#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[typeshare]
pub struct ExoCheckResult {
    pub index: u16, // just the index in the list of checks, to identify checks across Event
    pub state: CheckStatus,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Default, Clone, Debug)]
#[typeshare]
pub struct SessionStats {
    pub followers_count: u16,
    pub leaders_count: u16,
}

/// The protocol defines a set of valid actions that only clients can send
#[derive(Serialize, Deserialize, Clone, Debug, AsRefStr)]
#[serde(tag = "type", content = "content")]
#[typeshare]
pub enum Action {
    // Actions on sessions
    StartSession { name: String, group_id: String },
    StopSession, // the client_id will be used to verify the permission
    JoinSession { name: String, group_id: String },
    LeaveSession,
    GetSessions { group_id: String },

    // Code exo syncing
    SwitchExo { path: String },
    SendFile { path: String, content: String },
    SendResult { check_result: ExoCheckResult },
}

/// The server can send some events to one, some or all clients of the session
/// These events are generated after an action, in this case they are not necessarily sent to the
/// author of the action, but could sent to other clients.
/// These events can also be generated directly by the server (after some timeout or OS signal received)
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug, AsRefStr)]
#[serde(tag = "type", content = "content")]
#[typeshare]
pub enum Event {
    SessionStopped,
    SessionJoined(ClientNum),
    SessionLeaved,
    SessionsList(Vec<Session>),
    Stats(SessionStats),
    ServerStopped,
    ExoSwitched {
        path: String,
    },
    ForwardFile {
        client_num: ClientNum,
        file: ForwardedFile,
    },
    ForwardResult {
        client_num: ClientNum,
        result: ForwardedResult,
    },

    Error(LiveProtocolError),
}

/// An error sent from the server to clients after any message
/// that resolved in an error that is worth sending back to the client
#[derive(Serialize, Deserialize, Clone, Eq, PartialEq, Debug, AsRefStr)]
#[typeshare]
#[serde(tag = "type", content = "reason")]
pub enum LiveProtocolError {
    FailedToStartSession(String),
    FailedToJoinSession(String),
    FailedSendingWithoutSession,
    FailedToLeaveSession,
    SessionNotFound,
    CannotJoinOtherSession,
    ForbiddenSessionStop,
    ActionOnlyForLeader(String),
}

impl Display for LiveProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match &self {
            LiveProtocolError::FailedToStartSession(e) => format!("Failed to start a session: {e}"),
            LiveProtocolError::FailedToJoinSession(e) => format!("Failed to join the session: {e}"),
            LiveProtocolError::FailedToLeaveSession => {
                "No session joined, cannot leave the session.".to_string()
            }
            LiveProtocolError::FailedSendingWithoutSession => {
                "Failed to send file content or check result, because no session joined."
                    .to_string()
            }
            LiveProtocolError::SessionNotFound => "The session wasn't found.".to_string(),
            LiveProtocolError::CannotJoinOtherSession => {
                "You cannot join another session without having left your current session."
                    .to_string()
            }
            LiveProtocolError::ForbiddenSessionStop => {
                "You are not the creator of this session, you cannot stop it".to_string()
            }
            LiveProtocolError::ActionOnlyForLeader(action_name) => {
                format!("The action {action_name} is permitted to leaders of the session.")
            }
        };
        f.write_str(text.as_str())
    }
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[typeshare]
pub struct ForwardedFile {
    /// The relative path inside the exo folder, like "main.cpp", "src/main.rs", "lib/image.h"
    pub path: String,
    pub content: String,
    /// The time where this code was received on the server
    #[serde(with = "ts_seconds")]
    pub time: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Clone, Debug)]
#[typeshare]
pub struct ForwardedResult {
    /// The relative path inside the exo folder, like "main.cpp", "src/main.rs", "lib/image.h"
    pub check_result: ExoCheckResult,
    /// The time where this result was received on the server
    #[serde(with = "ts_seconds")]
    pub time: DateTime<Utc>,
}

// Implement serialisation and deserialisation strategy for Event and Action.
// Currently this is using JSON via serde_json
// TODO: how to avoid this annoying duplication ?? a macro ?
impl TryInto<Utf8Bytes> for Action {
    type Error = std::io::Error;
    fn try_into(self) -> Result<Utf8Bytes, std::io::Error> {
        let str = serde_json::to_string(&self).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Couldn't parse message: {e}"),
            )
        })?;
        Ok(Utf8Bytes::from(str))
    }
}

impl TryFrom<Utf8Bytes> for Action {
    type Error = std::io::Error;

    fn try_from(value: Utf8Bytes) -> Result<Action, std::io::Error> {
        serde_json::from_str::<Action>(&value).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Couldn't parse message: {e}"),
            )
        })
    }
}

impl TryInto<Utf8Bytes> for Event {
    type Error = std::io::Error;
    fn try_into(self) -> Result<Utf8Bytes, std::io::Error> {
        let str = serde_json::to_string(&self).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Couldn't parse message: {e}"),
            )
        })?;
        Ok(Utf8Bytes::from(str))
    }
}

impl TryFrom<Utf8Bytes> for Event {
    type Error = std::io::Error;

    fn try_from(value: Utf8Bytes) -> Result<Event, std::io::Error> {
        serde_json::from_str::<Event>(&value).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Couldn't parse message: {e}"),
            )
        })
    }
}
