use specta::Type;
use std::collections::HashMap;
use typeshare::typeshare;

use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{
    client::ClientRole,
    msg::{ClientNum, Event, SessionStats},
};

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

/// An action around a broadcasting need
#[derive(Debug)]
pub enum BroadcastAction {
    SendToLeaders(Event),
    SendToEveryone(Event),
    SaveClient(ClientRole, ClientNum, UnboundedSender<Event>),
    RemoveClient(ClientNum),
    Stop,
    SendStats, // only to leaders
}

/// This manager is running on the server in it's own tokio task. It maintains a list of Tx
/// for each ClientManager of the session and take care of broadcast to all clients or to leaders
pub struct SessionBroadcaster {
    /// A vector of transmitters to broadcast a message to clients in this session
    /// We also store the client role to filter leaders from the rest
    followers_broadcast_txs: HashMap<ClientNum, UnboundedSender<Event>>,
    leaders_broadcast_txs: HashMap<ClientNum, UnboundedSender<Event>>,

    /// A receiver to receive actions from the multiple ClientManager
    session_action_rx: UnboundedReceiver<BroadcastAction>,
}

impl SessionBroadcaster {
    pub fn new(rx: UnboundedReceiver<BroadcastAction>) -> Self {
        Self {
            followers_broadcast_txs: HashMap::new(),
            leaders_broadcast_txs: HashMap::new(),
            session_action_rx: rx,
        }
    }

    /// Run infinitely, until a Stop action is given, read SessionAction from
    /// a UnboundedReceiver<SessionAction> and react to each of them
    pub async fn run(&mut self) {
        while let Some(msg) = self.session_action_rx.recv().await {
            match msg {
                BroadcastAction::SendToLeaders(event) => self.broadcast(&event, true),
                BroadcastAction::SendToEveryone(event) => self.broadcast(&event, false),
                BroadcastAction::SaveClient(client_role, client_num, client_tx) => {
                    match client_role {
                        ClientRole::Follower => {
                            self.followers_broadcast_txs.insert(client_num, client_tx)
                        }
                        ClientRole::Leader => {
                            self.leaders_broadcast_txs.insert(client_num, client_tx)
                        }
                    };
                }
                BroadcastAction::RemoveClient(client_num) => {
                    // Try removing in followers, then leaders if the first fails
                    self.followers_broadcast_txs
                        .remove(&client_num)
                        .or_else(|| self.leaders_broadcast_txs.remove(&client_num));
                }
                BroadcastAction::Stop => break,
                BroadcastAction::SendStats => self.send_stats(),
            }
        }

        println!("One SessionBroadcaster is done");
    }

    /// Broadcast a message, to leaders only or everyone
    fn broadcast(&self, event: &Event, to_leaders_only: bool) {
        self.leaders_broadcast_txs.values().for_each(|tx| {
            let _ = tx.send(event.clone());
        });
        if !to_leaders_only {
            self.followers_broadcast_txs.values().for_each(|tx| {
                let _ = tx.send(event.clone());
            });
        };
    }

    /// Send basic statistics about the session to leaders
    /// This must be ran each time there is a change to the session
    fn send_stats(&self) {
        self.broadcast(
            &Event::Stats(SessionStats {
                followers_count: self.followers_broadcast_txs.len() as u16,
                leaders_count: self.leaders_broadcast_txs.len() as u16,
            }),
            true,
        );
    }
}
