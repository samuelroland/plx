// Client management on the server side of the live protocol
use futures_util::SinkExt;
use log::{info, warn};
use tokio::{
    select,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};
use tokio_stream::StreamExt;

use super::{client::ClientRole, msg::Msg};

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
    /// Allow to send a message to this client from the session manager
    pub external_write_rx: UnboundedReceiver<Msg>,
    /// A way to send Msg to the session manager if the message is authorized by the role
    /// That's an Option because the client will exist before creating/joining a session
    pub session_tx: Option<UnboundedSender<Msg>>,
    /// A way to interact with the server only to ask to create or join session
    pub server_tx: UnboundedSender<Msg>, //TODO: really ? not a mutex here as used very few ?
}

impl ClientManager {
    pub async fn run(&mut self) {
        select! {
            stream_el = self.websocket.next() => {
                match stream_el {
                    Some(Ok(msg)) => {
                        match Msg::from_ws_msg(&msg) {
                            Ok(Msg::DeleteSession { stop_secret }) => {
                                if self.role == ClientRole::Follower {
                                    warn!("Got a DeleteSession with a follower role: {}", msg);
                                    // that's a forged request, we can ignore it
                                }
                            },
                            Ok(Msg::CreateSession { name, group_id }) => {
                                // TODO
                                self.role = ClientRole::Leader
                            },
                            // Just forward the message to the session if it's a valid message
                            Ok(any_valid_msg) => {let _ = self.session_tx.as_ref().unwrap().send(any_valid_msg);}
                            Err(e) => info!("{}", e)
                        }
                    }
                    None => { let _ = self.websocket.close(None).await; }
                    _ => ()
                }
            }
            external_msg = self.external_write_rx.recv() => {
                if let Some(msg) = external_msg {
                    let _ = self.websocket.send(msg.into_ws_msg().unwrap()).await;
                }
            }
        };
    }
}

// enum ServerSessionAction {
//     CreateSession {
//         session: Session,
//         /// A way for the server to answer
//         get_back_session_tx: oneshot::Sender<UnboundedSender<Msg>>,
//     },
// }
