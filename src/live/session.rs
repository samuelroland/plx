use std::{collections::HashMap, time::SystemTime, vec};

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{
    client::ClientRole,
    msg::{ClientNum, Event, ForwardedFile, ForwardedResult},
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

/// An action on the session, can only be created by ClientManager
#[derive(Debug)]
pub struct SessionAction {
    pub client_tx: UnboundedSender<Event>,
    pub client_num: ClientNum,
    pub ctx: SessionActionCtx,
}

/// The action with some context
#[derive(Debug)]
pub enum SessionActionCtx {
    SendToLeaders(Event),
    SendToEveryone(Event),
    SaveClient(ClientRole),
    RemoveClient,
    Stop,
    SendStats, // only to leaders
}

/// This manager is running on the server in it's own tokio task and is responsible for
/// 1. forwarding messages to leaders or to all clients of the session
/// 2. handle the logic around session deletion
pub struct SessionManager {
    /// A vector of transmitters to broadcast a message to clients in this session
    /// We also store the client role to filter leaders from the rest
    broadcast_txs: HashMap<ClientNum, (ClientRole, UnboundedSender<Event>)>,

    /// A receiver to receive actions from the multiple ClientManager
    rx: UnboundedReceiver<SessionAction>,
}

impl SessionManager {
    pub fn new(rx: UnboundedReceiver<SessionAction>) -> Self {
        Self {
            broadcast_txs: HashMap::new(),
            rx,
        }
    }

    /// Run infinitely, until a Stop action is given, read SessionAction from
    /// a UnboundedReceiver<SessionAction> and react to each of them
    pub async fn run(&mut self) {
        println!("Starting SessionManager::run");
        while let Some(msg) = self.rx.recv().await {
            match msg.ctx {
                SessionActionCtx::SendToLeaders(event) => self.broadcast(&event, true),
                SessionActionCtx::SendToEveryone(event) => self.broadcast(&event, false),
                SessionActionCtx::SaveClient(client_role) => {
                    self.broadcast_txs
                        .insert(msg.client_num, (ClientRole::Follower, msg.client_tx));
                }
                SessionActionCtx::RemoveClient => {
                    self.broadcast_txs.remove(&msg.client_num);
                }
                SessionActionCtx::Stop => break,
                SessionActionCtx::SendStats => self.send_stats(),
            }
        }
    }

    /// Broadcast a message, to leaders only or everyone
    fn broadcast(&self, event: &Event, to_leaders_only: bool) {
        let txs: Vec<&UnboundedSender<Event>> = if to_leaders_only {
            self.broadcast_txs
                .iter()
                .filter_map(|(_, (role, tx))| {
                    if *role == ClientRole::Leader {
                        Some(tx)
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            self.broadcast_txs
                .iter()
                .map(|(_, (role, tx))| tx)
                .collect()
        };

        txs.iter().for_each(|tx| {
            let _ = tx.send(event.clone());
        });
    }

    /// Send basic statistics about the session to leaders
    /// This must be ran each time there is a change to the session
    fn send_stats(&self) {
        let (followers_iter, leaders_iter): (Vec<_>, Vec<_>) = self
            .broadcast_txs
            .iter()
            .partition(|(_, (role, tx))| *role == ClientRole::Follower);

        self.broadcast(
            &Event::Stats {
                followers_count: followers_iter.len() as u16,
                leaders_count: leaders_iter.len() as u16,
            },
            true,
        );
    }
}
