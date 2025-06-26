use std::sync::Arc;

use crate::live::msg::LiveProtocolError;

// Client management on the server side of the live protocol
use super::{
    client::ClientRole,
    msg::{Action, ClientNum, Event},
    session::{Session, SessionAction, SessionActionCtx, SessionManager},
    sessions_manager::{self, SessionsManager},
};
use futures_util::{stream::FusedStream, SinkExt};
use log::{info, warn};
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::{Error, Message};

/// Manager of a connected client on the server, it owns the websocket connexion
/// It is responsible to wait on the websocket.read() and external_write_rx.read()
/// If there is an external write, just send it,
/// if there is a message from the client, parse it, verify the message has the permission
/// to be sent and send it to the SessionManager
pub struct ClientManager {
    /// A unique ID sent by the client during websocket handshake
    pub client_id: String,
    pub role: ClientRole,
    /// The WebSocket where the client is connected, on which we can send() or read()
    pub websocket: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    /// A way to interact with the server only to ask to create or join session
    // pub server_tx: UnboundedSender<Msg>, //TODO: really ? not a mutex here as used very few ?
    pub session: Option<SessionLink>,

    pub sessions_manager: Arc<SessionsManager>,
}

/// Keeping a link to the session via message passing in both directions
pub struct SessionLink {
    /// Allow to send a message from the session manager to this client
    pub client_rx: UnboundedReceiver<Event>,
    /// Keep a copy of the tx so we can pass it to SessionAction
    pub client_tx: UnboundedSender<Event>,
    /// Client number attribued by SessionsManager::start_session()
    pub client_num: ClientNum,
    /// A way to send messages to the session manager if the message is authorized by the role
    /// That's an Option because the client will exist before creating/joining a session
    /// If it is None, it means the client hasn't joined a session yet
    pub session_tx: UnboundedSender<SessionAction>,
}

impl ClientManager {
    pub async fn run(&mut self) {
        loop {
            if self.websocket.is_terminated() {
                break;
            }
            // We have to check if the session exist at each round,
            // because each message could have closed the session and set it to None
            match &mut self.session {
                // Just listen on the websocket stream to wait for GetSessions / JoinSession messages
                None => {
                    let stream_el = self.websocket.next().await;
                    self.handle_msg(stream_el).await;
                }
                // If there is a session, listen on both streams
                Some(session) => {
                    select! {
                        stream_el = self.websocket.next() => {
                            self.handle_msg(stream_el).await;
                        }
                        external_msg = session.client_rx.recv() => {
                            match external_msg {
                                Some(event) => {self.websocket.send(Message::Text(event.try_into().unwrap()));}
                                None => {self.session = None;}
                            }
                        }
                    };
                }
            }
        }
    }

    async fn handle_msg(&mut self, stream_element: Option<Result<Message, Error>>) {
        match stream_element {
            Some(Ok(msg)) => {
                match Action::try_from(msg.into_text().unwrap()) {
                    Ok(Action::StartSession { name, group_id }) => {
                        // TODO
                        // TODO: check name and group validity
                        // TODO: check name + group uniqueness
                        let (client_tx, client_rx) =
                            tokio::sync::mpsc::unbounded_channel::<Event>();
                        match self.sessions_manager.start_session(
                            name,
                            group_id,
                            self.client_id,
                            client_tx,
                        ) {
                            Ok((client_num, session_tx)) => {
                                self.session = Some({
                                    SessionLink {
                                        client_rx,
                                        client_tx,
                                        client_num,
                                        session_tx,
                                    }
                                });
                                info!("Session created");

                                self.role = ClientRole::Leader;
                                client_tx.send(Event::SessionStarted);
                            }
                            Err(e) => {
                                self.websocket.send(Message::Text(
                                    Event::Error(LiveProtocolError::FailedToStartSession(e))
                                        .try_into()
                                        .unwrap(),
                                ));
                            }
                        }
                    }
                    Ok(Action::StopSession) => {
                        if self.role == ClientRole::Follower {
                            warn!("Got a DeleteSession with a follower role: {}", msg);
                            // that's a forged request, we can ignore it
                        }
                        if let Some(session) = &self.session {
                            let _ = session.session_tx.send(SessionAction {
                                ctx: SessionActionCtx::Stop,
                            });
                            self.role = ClientRole::Follower;
                        }
                        // Do not touch self.session for now, wait for the session_manager choosing
                        // to stop itself via SessionStopped message
                    }
                    Ok(Action::JoinSession { name, group_id }) => match &self.session {
                        Some(session) => {
                            let _ = self
                                .websocket
                                .send(Message::Text(
                                    Event::Error(LiveProtocolError::CannotJoinOtherSession)
                                        .try_into()
                                        .unwrap(),
                                ))
                                .await;
                        }
                        None => {
                            let (client_tx, client_rx) =
                                tokio::sync::mpsc::unbounded_channel::<Event>();
                            match self
                                .sessions_manager
                                .join_session(name, group_id, client_tx)
                            {
                                Ok((client_num, session_tx)) => {
                                    self.session = Some(SessionLink {
                                        client_rx,
                                        client_tx,
                                        client_num,
                                        session_tx,
                                    })
                                }
                                Err(e) => {
                                    self.websocket.send(Message::Text(
                                        Event::Error(LiveProtocolError::FailedToJoinSession(e))
                                            .try_into()
                                            .unwrap(),
                                    ));
                                }
                            }
                        }
                    },

                    Ok(Action::LeaveSession) => match self.session {
                        Some(session) => self
                            .sessions_manager
                            .leave_session(session.client_num, session.session_tx),
                        None => {
                            self.websocket.send(Message::Text(
                                Event::Error(LiveProtocolError::FailedToLeaveSession)
                                    .try_into()
                                    .unwrap(),
                            ));
                        }
                    },

                    Ok(Action::GetSessions { group_id }) => {
                        self.websocket.send(Message::Text(
                            Event::SessionsList(self.sessions_manager.get_sessions(&group_id))
                                .try_into()
                                .unwrap(),
                        ));
                    }

                    Ok(Action::SendFile { file, content }) => {
                        self.websocket.send(Message::Text(
                            Event::SessionsList(self.sessions_manager.get_sessions(&group_id))
                                .try_into()
                                .unwrap(),
                        ));
                    }
                    // Just forward the message to the session if it's a valid message
                    // Ok(any_valid_msg) => {
                    //     if let Some(session) = &self.session {
                    //         let _ = session.session_tx.send(any_valid_msg);
                    //     }
                    // }
                    Err(e) => {
                        info!("{}", e)
                    }
                }
            }
            None => {
                let _ = self.websocket.close(None).await;
            }
            _ => (),
        }
    }
}

// enum ServerSessionAction {
//     CreateSession {
//         session: Session,
//         /// A way for the server to answer
//         get_back_session_tx: oneshot::Sender<UnboundedSender<Msg>>,
//     },
// }
