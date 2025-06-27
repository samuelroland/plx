use std::{collections::HashMap, ops::Deref, sync::Arc, time::SystemTime, vec};

use log::error;
use tokio::sync::RwLock;

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::msg::LiveProtocolError;
use super::{
    client::ClientRole,
    msg::{ClientNum, Event},
    session::{BroadcastAction, Session, SessionBroadcaster},
};

struct SessionState {
    session: Session,
    /// The leader id of the leader that created this session, this is the only client that is
    /// authorized to close a session, the potential other leaders cannot do that.
    leader_client_id: String,
    last_attributed_client_num: ClientNum,
    /// A copy of the sender to give to new clients when joining
    tx: UnboundedSender<BroadcastAction>,
}
// The first key is the group_id, the second is the session name, the u32 is the last client_num used
type Session2DMap = HashMap<String, HashMap<String, SessionState>>;

pub struct SessionsManager {
    /// Keep a 2 dimensionnal hashmap of all sessions, indexed by session's group id, then session's name, to finally access a SessionState
    sessions_by_group_and_name: RwLock<Session2DMap>,
    /// Also store the mapping between leader client_id and a copy of Session to easily find the SessionState in self.sessions
    sessions_info_by_leaders_id: RwLock<HashMap<String, Session>>,
}

impl SessionsManager {
    pub fn new() -> Self {
        Self {
            sessions_by_group_and_name: RwLock::new(Session2DMap::new()),
            sessions_info_by_leaders_id: RwLock::new(HashMap::new()),
        }
    }

    pub async fn start_session(
        &self,
        name: String,
        group_id: String,
        leader_client_id: String,
        client_tx: UnboundedSender<Event>,
    ) -> Result<(ClientNum, UnboundedSender<BroadcastAction>), String> {
        let (session_tx, session_rx) = tokio::sync::mpsc::unbounded_channel::<BroadcastAction>();
        let mut session_broadcaster = SessionBroadcaster::new(session_rx);
        let leaders_client_num = ClientNum(1);
        {
            if self
                .sessions_by_group_and_name
                .read()
                .await
                .get(&group_id)
                .and_then(|subhashmap| subhashmap.get(&name))
                .is_some()
            {
                return Err(
                    "There is already a session with the same group id and name combination."
                        .to_string(),
                );
            }
        }
        tokio::spawn(async move {
            session_broadcaster.run().await;
        });

        let _ = session_tx.send(BroadcastAction::SaveClient(
            ClientRole::Leader,
            leaders_client_num.clone(),
            client_tx,
        ));

        let session_info = Session {
            name: name.clone(),
            group_id: group_id.clone(),
        };
        let session_state = SessionState {
            session: session_info.clone(),
            leader_client_id: leader_client_id.clone(),
            last_attributed_client_num: leaders_client_num.clone(),
            tx: session_tx.clone(),
        };

        {
            self.sessions_by_group_and_name
                .write()
                .await
                .entry(group_id)
                .or_default()
                .insert(name, session_state);
        }
        {
            self.sessions_info_by_leaders_id
                .write()
                .await
                .insert(leader_client_id, session_info);
        }
        Ok((leaders_client_num, session_tx))
    }

    pub async fn join_session(
        &self,
        name: String,
        group_id: String,
        client_tx: UnboundedSender<Event>,
    ) -> Result<(ClientNum, UnboundedSender<BroadcastAction>), String> {
        let mut write_guard = self.sessions_by_group_and_name.write().await;
        let session = write_guard
            .get_mut(&group_id)
            .ok_or("No session found with this group id")?
            .get_mut(&name)
            .ok_or("No session found with this name in this group id")?;
        let new_client_num = ClientNum(session.last_attributed_client_num.0 + 1);
        session.last_attributed_client_num = new_client_num.clone();
        let session_tx = session.tx.clone();
        drop(write_guard);
        let _ = session_tx.send(BroadcastAction::SaveClient(
            ClientRole::Follower,
            new_client_num.clone(),
            client_tx.clone(),
        ));
        let _ = client_tx.send(Event::SessionJoined);
        let _ = session_tx.send(BroadcastAction::SendStats);

        Ok((new_client_num, session_tx))
    }

    pub fn leave_session(
        &self,
        client_num: ClientNum,
        session_tx: UnboundedSender<BroadcastAction>,
    ) {
        let _ = session_tx.send(BroadcastAction::RemoveClient(client_num));
        let _ = session_tx.send(BroadcastAction::SendStats);
    }

    pub async fn get_sessions(&self, group_id: &String) -> Vec<Session> {
        let read_guard = self.sessions_by_group_and_name.read().await;

        match read_guard.get(group_id) {
            Some(group) => group
                .iter()
                .map(|(name, state)| state.session.clone())
                .collect(),
            None => vec![],
        }
    }

    pub async fn stop_session(&self, client_id: &String) -> Result<(), LiveProtocolError> {
        let reader2 = self.sessions_info_by_leaders_id.read().await;

        let session = reader2
            .get(client_id)
            .ok_or(LiveProtocolError::ForbiddenSessionStop)?
            .clone();
        drop(reader2);

        let reader = self.sessions_by_group_and_name.read().await;
        // Make sure the session actually exist and that the client is the leader creator of the session
        if let Some(session_state) = reader
            .get(&session.group_id)
            .and_then(|h| h.get(&session.name))
        {
            if session_state.leader_client_id == *client_id {
                let _ = session_state
                    .tx
                    .send(BroadcastAction::SendToEveryone(Event::SessionStopped));
                let _ = session_state.tx.send(BroadcastAction::Stop);
                drop(reader);
                {
                    self.sessions_by_group_and_name
                        .write()
                        .await
                        .get_mut(&session.group_id)
                        .and_then(|h| h.remove(&session.name));
                }
                {
                    self.sessions_info_by_leaders_id
                        .write()
                        .await
                        .remove(client_id);
                }

                Ok(())
            } else {
                Err(LiveProtocolError::ForbiddenSessionStop)
            }
        } else {
            error!("ClientManager.session contains a session that doesn't exist");
            Ok(()) // just ignore the problem and considere the session to be already closed
        }
    }
}
