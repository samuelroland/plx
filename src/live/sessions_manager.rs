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
        let session_state = SessionState {
            session: Session { name, group_id },
            leader_client_id,
            last_attributed_client_num: leaders_client_num.clone(),
            tx: session_tx.clone(),
        };

        tokio::spawn(async move {
            session_manager.run().await;
        });
        let add_leader_action = SessionAction {
            client_tx: client_tx.clone(),
            client_num: leaders_client_num.clone(),
            ctx: SessionActionCtx::SaveClient(ClientRole::Leader),
        };
        session_tx.send(add_leader_action);
        client_tx.send(Event::SessionStarted);

        Ok((leaders_client_num, session_tx))
    }

    pub fn stop_session() {
        todo!()
    }
}
