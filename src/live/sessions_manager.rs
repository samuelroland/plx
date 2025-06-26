use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, RwLock},
    time::SystemTime,
    vec,
};

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use super::{
    client::ClientRole,
    msg::{ClientNum, Event},
    session::{Session, SessionAction, SessionActionCtx, SessionManager},
};

struct SessionState {
    session: Session,
    /// The leader id of the leader that created this session, this is the only client that is
    /// authorized to close a session, the potential other leaders cannot do that.
    leader_client_id: String,
    last_attributed_client_num: ClientNum,
    /// A copy of the sender to give to new clients joining
    tx: UnboundedSender<SessionAction>,
}
// The first key is the group_id, the second is the session name, the u32 is the last client_num used
type Session2DMap = HashMap<String, HashMap<String, SessionState>>;

pub struct SessionsManager {
    sessions: RwLock<Session2DMap>,
}

impl SessionsManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(Session2DMap::new()),
        }
    }

    pub fn start_session(
        &self,
        name: String,
        group_id: String,
        leader_client_id: String,
        client_tx: UnboundedSender<Event>,
    ) -> Result<(ClientNum, UnboundedSender<SessionAction>), String> {
        let (session_tx, session_rx) = tokio::sync::mpsc::unbounded_channel::<SessionAction>();
        let mut session_manager = SessionManager::new(session_rx);
        let leaders_client_num = ClientNum(1);
        {
            if self
                .sessions
                .read()
                .unwrap()
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
            session_manager.run().await;
        });
        let add_leader_action = SessionAction {
            ctx: SessionActionCtx::SaveClient(
                ClientRole::Leader,
                leaders_client_num.clone(),
                client_tx,
            ),
        };
        session_tx.send(add_leader_action);

        let session_state = SessionState {
            session: Session {
                name: name.clone(),
                group_id: group_id.clone(),
            },
            leader_client_id,
            last_attributed_client_num: leaders_client_num.clone(),
            tx: session_tx.clone(),
        };

        {
            self.sessions
                .write()
                .unwrap()
                .entry(group_id)
                .or_insert(HashMap::new())
                .insert(name, session_state);
        }
        Ok((leaders_client_num, session_tx))
    }

    pub fn join_session(
        &self,
        name: String,
        group_id: String,
        client_tx: UnboundedSender<Event>,
    ) -> Result<(ClientNum, UnboundedSender<SessionAction>), String> {
        let mut write_guard = self.sessions.write().unwrap();
        let session = write_guard
            .get_mut(&group_id)
            .ok_or("No session found with this group id")?
            .get_mut(&name)
            .ok_or("No session found with this name in this group id")?;
        let new_client_num = ClientNum(session.last_attributed_client_num.0 + 1);
        session.last_attributed_client_num = new_client_num.clone();
        let session_tx = session.tx.clone();
        drop(write_guard);
        let save_client_action = SessionAction {
            ctx: SessionActionCtx::SaveClient(
                ClientRole::Leader,
                new_client_num.clone(),
                client_tx,
            ),
        };
        session_tx.send(save_client_action);

        Ok((new_client_num, session_tx))
    }

    pub fn leave_session(&self, client_num: ClientNum, session_tx: UnboundedSender<SessionAction>) {
        let action = SessionAction {
            ctx: SessionActionCtx::RemoveClient(client_num),
        };
        session_tx.send(action);
    }

    pub fn get_sessions(&self, group_id: &String) -> Vec<Session> {
        let read_guard = self.sessions.read().unwrap();

        match read_guard.get(group_id) {
            Some(group) => group
                .iter()
                .map(|(name, state)| state.session.clone())
                .collect(),
            None => vec![],
        }
    }

    pub fn stop_session() {
        todo!()
    }
}
