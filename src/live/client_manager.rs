use crate::live::msg::LiveProtocolError;

// Client management on the server side of the live protocol
use super::{client::ClientRole, msg::Msg};
use futures_util::SinkExt;
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
    pub session: Option<SessionMessagePassing>,
}

pub struct SessionMessagePassing {
    /// Allow to send a message to this client from the session manager
    pub external_write_rx: UnboundedReceiver<Msg>,
    /// A way to send Msg to the session manager if the message is authorized by the role
    /// That's an Option because the client will exist before creating/joining a session
    pub session_tx: UnboundedSender<Msg>,
}

impl ClientManager {
    pub async fn run(&mut self) {
        loop {
            match &mut self.session {
                // Just listen on the websocket stream to wait for Get JoinSession message
                None => {
                    let stream_el = self.websocket.next().await;
                    self.handle_msg(stream_el);
                }
                // If there is a session, listen on both streams
                Some(session) => {
                    select! {
                        stream_el = self.websocket.next() => {
                            self.handle_msg(stream_el);
                        }
                        external_msg = session.external_write_rx.recv() => {
                            if let Some(session) = &self.session {
                                    if let Some(msg) = external_msg {
                                    let _ = self.websocket.send(msg.into_ws_msg().unwrap()).await;
                                }
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
                match Msg::from_ws_msg(&msg) {
                    Ok(Msg::DeleteSession {}) => {
                        if self.role == ClientRole::Follower {
                            warn!("Got a DeleteSession with a follower role: {}", msg);
                            // that's a forged request, we can ignore it
                        }
                        self.session = None;
                    }
                    Ok(Msg::CreateSession { name, group_id }) => {
                        // TODO
                        self.role = ClientRole::Leader;
                    }
                    Ok(Msg::JoinSession { name, group_id }) => {
                        if self.session.is_some() {
                            self.websocket.send(
                                Msg::Error(LiveProtocolError::CannotJoinOtherSession)
                                    .into_ws_msg()
                                    .unwrap(),
                            );
                        }
                    }
                    // Just forward the message to the session if it's a valid message
                    Ok(any_valid_msg) => {
                        if let Some(session) = &self.session {
                            let _ = session.session_tx.send(any_valid_msg);
                        }
                    }
                    Err(e) => info!("{}", e),
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
