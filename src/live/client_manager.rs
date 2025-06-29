use std::{sync::Arc, time::SystemTime};

use crate::live::msg::LiveProtocolError;

// Client management on the server side of the live protocol
use super::{
    client::ClientRole,
    msg::{Action, ClientNum, Event, ForwardedFile, ForwardedResult},
    session::{BroadcastAction, Session, SessionBroadcaster},
    sessions_manager::{self, SessionsManager},
};
use chrono::{DateTime, Utc};
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
    /// The WebSocket where the client is connected, on which we can send() or read()
    pub websocket: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    pub session: Option<SessionLink>,
    pub sessions_manager: Arc<SessionsManager>,
}

/// Keeping a link to the session via message passing in both directions
pub struct SessionLink {
    /// Allow to send a message from the session manager to this client
    pub client_rx: UnboundedReceiver<Event>,
    /// Keep a copy of the tx so we can pass it to SessionAction
    pub client_tx: UnboundedSender<Event>,
    pub role: ClientRole,
    /// Client number attributed by SessionsManager::start_session()
    pub client_num: ClientNum,
    /// A way to send messages to the session manager if the message is authorized by the role
    /// That's an Option because the client will exist before creating/joining a session
    /// If it is None, it means the client hasn't joined a session yet
    pub session_tx: UnboundedSender<BroadcastAction>,
}

impl ClientManager {
    pub async fn run(&mut self) {
        println!("ClientManager for new client");
        loop {
            // Stop here if client disconnect themself
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
                                Some(Event::SessionStopped) => {
                                    self.session = None;
                                    self.send_event(Event::SessionStopped).await;
                                }
                                Some(Event::ServerStopped) => {
                                    self.session = None;
                                    self.send_event(Event::SessionStopped).await;
                                    self.send_event(Event::ServerStopped).await;
                                    let _ = self.websocket.close(None).await; // TODO: should we also read() again until we get Error::ConnectionClosed ?
                                    // https://docs.rs/tungstenite/latest/tungstenite/protocol/struct.WebSocket.html#method.close
                                    break; // stops itself too
                                }
                                Some(event) => {self.send_event(event).await;}
                                None => {self.session = None;}
                            }
                        }
                    };
                }
            }
        }

        println!("ClientManager for client_id '{}' is done", self.client_id);
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
                        match self
                            .sessions_manager
                            .start_session(
                                name,
                                group_id,
                                self.client_id.clone(),
                                client_tx.clone(),
                            )
                            .await
                        {
                            Ok((client_num, session_tx)) => {
                                self.session = Some({
                                    SessionLink {
                                        client_rx,
                                        client_tx: client_tx.clone(),
                                        client_num,
                                        role: ClientRole::Leader,
                                        session_tx,
                                    }
                                });
                                info!("Session created");

                                let _ = client_tx.send(Event::SessionStarted);
                            }
                            Err(e) => {
                                self.send_error(LiveProtocolError::FailedToStartSession(e))
                                    .await;
                            }
                        }
                    }
                    Ok(Action::StopSession) => {
                        if let Some(session) = &self.session {
                            if session.role == ClientRole::Follower {
                                self.send_error(LiveProtocolError::ForbiddenSessionStop)
                                    .await;
                            } else if let Some(session) = &self.session {
                                let result =
                                    self.sessions_manager.stop_session(&self.client_id).await;
                                if let Err(e) = result {
                                    self.send_error(e).await;
                                }
                                // Do not touch self.session for now, wait on the SessionStopped message
                                // sent by the session_broadcaster choosing after it stopped itself
                            }
                        } else {
                            self.send_error(LiveProtocolError::SessionNotFound).await;
                        }
                    }
                    Ok(Action::JoinSession { name, group_id }) => match &self.session {
                        Some(session) => {
                            self.send_error(LiveProtocolError::CannotJoinOtherSession)
                                .await;
                        }
                        None => {
                            let (client_tx, client_rx) =
                                tokio::sync::mpsc::unbounded_channel::<Event>();
                            match self
                                .sessions_manager
                                .join_session(
                                    name,
                                    group_id,
                                    self.client_id.clone(),
                                    client_tx.clone(),
                                )
                                .await
                            {
                                Ok((client_num, role, session_tx)) => {
                                    self.session = Some(SessionLink {
                                        client_rx,
                                        client_tx,
                                        client_num,
                                        role,
                                        session_tx,
                                    })
                                }
                                Err(e) => {
                                    self.send_error(LiveProtocolError::FailedToJoinSession(e))
                                        .await;
                                }
                            }
                        }
                    },

                    Ok(Action::LeaveSession) => match &self.session {
                        Some(session) => {
                            self.sessions_manager.leave_session(
                                session.client_num.clone(),
                                session.session_tx.clone(),
                            );
                            self.session = None;
                        }
                        None => {
                            self.send_error(LiveProtocolError::FailedToLeaveSession)
                                .await;
                        }
                    },

                    Ok(Action::GetSessions { group_id }) => {
                        self.send_event(Event::SessionsList(
                            self.sessions_manager.get_sessions(&group_id).await,
                        ))
                        .await;
                    }

                    Ok(Action::SendFile { file, content }) => match &self.session {
                        Some(session) => {
                            let _ = session.session_tx.send(BroadcastAction::SendToLeaders(
                                Event::ForwardFile(
                                    session.client_num.clone(),
                                    ForwardedFile {
                                        file,
                                        content,
                                        time: DateTime::<Utc>::from(SystemTime::now()),
                                    },
                                ),
                            ));
                        }
                        None => {
                            self.send_error(LiveProtocolError::FailedSendingWithoutSession)
                                .await;
                        }
                    },
                    Ok(Action::SendResult { check_result }) => match &self.session {
                        Some(session) => {
                            let _ = session.session_tx.send(BroadcastAction::SendToLeaders(
                                Event::ForwardResult(
                                    session.client_num.clone(),
                                    ForwardedResult {
                                        check_result,
                                        time: DateTime::<Utc>::from(SystemTime::now()),
                                    },
                                ),
                            ));
                        }
                        None => {
                            self.send_error(LiveProtocolError::FailedSendingWithoutSession)
                                .await;
                        }
                    },
                    Ok(Action::ExoSwitch { path }) => {
                        if let Some(session) = &self.session {
                            if session.role == ClientRole::Follower {
                                self.send_error(LiveProtocolError::ActionOnlyForLeader(
                                    "switch of exo".to_string(),
                                ))
                                .await;
                            } else {
                                let _ = session.session_tx.send(BroadcastAction::SendToEveryone(
                                    Event::ExoSwitched { path },
                                ));
                            }
                        } else {
                            self.send_error(LiveProtocolError::SessionNotFound).await;
                        }
                    }
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

    async fn send_event(&mut self, event: Event) {
        let _ = self
            .websocket
            .send(Message::Text(event.try_into().unwrap()))
            .await;
    }
    async fn send_error(&mut self, error: LiveProtocolError) {
        self.send_event(Event::Error(error)).await;
    }
}
