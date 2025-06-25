use std::{ops::Deref, time::SystemTime, vec};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{
    client::ClientRole,
    msg::{ClientNum, ForwardedFile, ForwardedResult, Msg},
};

/// A live session, this is the representation sent to clients
/// when listing all sessions or after session creation
#[derive(Serialize, Deserialize, Clone, Debug)]
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

/// This manager is running on the server in it's own tokio task and is responsible for
/// 1. forwarding messages to leaders or to all clients of the session
/// 2. handle the logic around session deletion
pub struct SessionManager {
    session: Session,
    /// The leader id of the leader that created this session, this is the only client that is
    /// authorized to close a session, the potential other leaders cannot do that.
    leader_client_id: String,

    /// A vector of transmitters to broadcast a message to clients in this session
    /// We also store the client role to filter leaders from the rest
    broadcast_txs: Vec<(ClientRole, UnboundedSender<Msg>)>,

    /// A receiver to receive messages from the tasks running ClientManager
    rx: UnboundedReceiver<Msg>,
}

impl SessionManager {
    pub fn new(
        session: Session,
        leader_client_id: String,
        rx: UnboundedReceiver<Msg>,
        leader_tx: UnboundedSender<Msg>,
    ) -> Self {
        Self {
            session,
            leader_client_id,
            broadcast_txs: vec![(ClientRole::Leader, leader_tx)],
            rx,
        }
    }

    pub async fn run(&mut self) {
        println!("Starting SessionManager::run");
        while let Some(msg) = self.rx.recv().await {
            println!("Got a message {msg:?}");
            match msg {
                // Session management

                Msg::StopSession => {
                    self.broadcast(&Msg::SessionStopped, false); // Sending this specific message
                                                                 // TODO: remove the session from global list
                    break;
                    // all the attributes should be dropped, the websocket connections will be closed in each ClientManager's task
                }
                Msg::GetSessions { group_id } => {

                }
                Msg::JoinSession { name, group_id } => todo!(),
                Msg::LeaveSession => todo!(),

                // Code exos
                Msg::SendFile { file, content } => self.broadcast(
                    &Msg::ForwardFile(
                        ClientNum(2), // TODO: fix
                        ForwardedFile {
                            file,
                            content,
                            time: SystemTime::now(),
                        },
                    ),
                    true,
                ),
                Msg::SendResult { check_id, passed } => self.broadcast(
                    &Msg::ForwardResult(
                        ClientNum(2), // TODO: fix
                        ForwardedResult {
                            check_id,
                            passed,
                            time: SystemTime::now(),
                        },
                    ),
                    true,
                ),

                // Note: do not use "_ =>" to make sure we decide what to do on new message types

                // Everything that's invalid in this context is just ignored
                // It's the case for any message that only the server can send
                Msg::StartSession {..} // already managed in ClientManager
                | Msg::SessionStarted
                | Msg::SessionStopped
                | Msg::SessionJoined
                | Msg::ForwardFile(..)
                | Msg::ForwardResult(..)
                | Msg::Stats { .. } => {}
                Msg::Error(..) => {}
            }
        }
    }

    /// Broadcast a message, to leaders only or everyone
    fn broadcast(&self, msg: &Msg, to_leaders_only: bool) {
        let txs: Vec<&UnboundedSender<Msg>> = if to_leaders_only {
            self.broadcast_txs
                .iter()
                .filter_map(|(role, tx)| {
                    if *role == ClientRole::Leader {
                        Some(tx)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            self.broadcast_txs.iter().map(|(role, tx)| tx).collect()
        };

        txs.iter().for_each(|tx| {
            let _ = tx.send(msg.clone());
        });
    }

    /// Send basic statistics about the session to leaders
    /// This must be ran each time there is a change to the session
    fn send_stats(&self) {
        let (followers_iter, leaders_iter): (Vec<_>, Vec<_>) = self
            .broadcast_txs
            .iter()
            .partition(|(role, tx)| *role == ClientRole::Follower);

        self.broadcast(
            &Msg::Stats {
                followers_count: followers_iter.len() as u16,
                leaders_count: leaders_iter.len() as u16,
            },
            true,
        );
    }
}
