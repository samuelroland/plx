use std::{
    ptr::NonNull,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
    vec,
};

use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::{
    io::AsyncWriteExt,
    net::{TcpSocket, TcpStream},
    runtime::Runtime,
    sync::{
        mpsc::{self, UnboundedReceiver, UnboundedSender},
        oneshot,
    },
};

use futures_util::{select, stream::StreamExt, SinkExt};
use tokio::net::TcpListener;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes, WebSocket};

// Default port of the live protocol
pub const DEFAULT_LIVE_PORT: u16 = 9120;

/// A live session
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

/// The live server serving live sessions
pub struct LiveServer {
    // /// The single thread used to accept TCP connection
    // accept_thread: JoinHandle<()>,
    // /// A list of thread waiting on clients messages, one thread per connected client
    // client_threads: Arc<Mutex<Vec<JoinHandle<()>>>>,
    runtime: Runtime,
}

/// Internal connected client struct
/// It is responsible to wait on the websocket.read() and external_write_rx.read()
/// If there is an external write, just send it, if there is a message from the client
/// we verify the message has the right to be sent
struct Client {
    client_id: String,
    role: ClientRole,
    /// The WebSocket where the client is connected, on which we can send() or read()
    websocket: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    /// Allow to send a message to this client from the session manager
    external_write_rx: UnboundedReceiver<Msg>,
    /// A way to send Msg to the session manager if the message is authorized by the role
    /// That's an Option because the client will exist before creating/joining a session
    session_tx: Option<UnboundedSender<Msg>>,
    /// A way to interact with the server only to ask to create or join session
    server_tx: UnboundedSender<Msg>, //TODO: really ? not a mutex here as used very few ?
}

impl Client {
    async fn run(&mut self) {
        select! {
            stream_el = self.websocket.next() => {
                match stream_el {
                    Some(Ok(msg)) => {
                        match Msg::from_ws_msg(&msg) {
                            Ok(Msg::Init { client_id }) => {},
                            Ok(Msg::DeleteSession { stop_secret }) => {
                                if self.role == ClientRole::Follower {
                                    warn!("Got a DeleteSession with a follower role: {}", msg);
                                    return; // that's a forged request,
                                }
                            },
                            Ok(Msg::CreateSession { name, group_id }) => {
                                // TODO
                                self.role = ClientRole::Leader
                            },
                            // Just forward the message to the session if it's a valid message
                            Ok(any_valid_msg) => {self.session_tx.as_ref().unwrap().send(any_valid_msg);}
                            Err(e) => info!("{}", e)
                        }
                    }
                    None => { self.websocket.close(None); }
                    _ => ()
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

#[derive(Eq, PartialEq)]
enum ClientRole {
    /// Default role, for anyone following a session
    Follower,
    /// When the client creates a session, it becames a leader client
    Leader,
}

#[derive(Serialize, Deserialize)]
enum Msg {
    Init { client_id: String },
    CreateSession { name: String, group_id: String },
    DeleteSession { stop_secret: String },
    GetSessions { group_id: String },
    SendCode { file: String, content: String },
    SendResult { check_id: u32, passed: bool },
}

impl Msg {
    pub fn from_ws_msg(ws_msg: &Message) -> Result<Msg, String> {
        match ws_msg {
            Message::Text(utf8) => serde_json::from_str::<Msg>(&utf8)
                .map_err(|e| format!("Couldn't parse message: {e}")),
            _ => Err("Message was not in Text format".to_string()),
        }
    }
}

impl LiveServer {
    /// Create a live server and prepare a Tokio runtime for
    pub fn new() -> Result<Self, std::io::Error> {
        let runtime = tokio::runtime::Builder::new_current_thread().build()?;

        Ok(LiveServer { runtime })
    }

    /// Start a server listening on given port, use the DEFAULT_LIVE_PORT or a custom one
    /// listening on all network interfaces ("0.0.0.0") to be publicly accessible
    /// This function is blocking and will never stop, except when calling stop()
    pub fn start(&self, port: u16) {
        self.runtime.block_on(async {
            let (tx, rx) = mpsc::unbounded_channel::<Msg>();
            // Start binding here, so it can fail if the port is already used.
            let listener = TcpListener::bind(format!("0.0.0.0:{}", DEFAULT_LIVE_PORT))
                .await
                .unwrap();

            // On all new TCP connections, just spawn a new task to process the new client
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(Self::process_client(stream));
            }
        });
    }

    /// Once a TcpSocket has been accepted into a TcpStream, we can start the websocket connection
    async fn process_client(stream: TcpStream) {
        // TODO: check version number, maybe via accept_hdr ?

        // Accept the websocket stream with websocket handshake
        match tokio_tungstenite::accept_async(stream).await {
            Ok(mut websocket) => {
                // Wait on the Init message
                let first_msg = websocket.next().await;
                if let Some(Ok(Message::Text(text))) = first_msg {
                    match serde_json::from_str::<Msg>(text.as_str()) {
                        Ok(Msg::Init { client_id }) => {
                            let client = Client {
                                client_id,
                                role: ClientRole::Follower,
                                websocket,
                                external_write_rx: todo!(),
                                session_tx: todo!(),
                                server_tx: todo!(),
                            };
                            client.run();
                        }
                        // Ignore failed init message, we don't want to spend time sending responses to
                        // invalid clients at this point, this is probably spam
                        // TODO: good idea ?
                        _ => {
                            info!("Client failed to Init");
                            websocket.close(None).await;
                        }
                    }
                }
                let a = websocket.next().await;
            }
            Err(e) => warn!("Got a handshake error: {}", e.to_string()),
        }
    }

    /// Stopping the server by stopping the runtime
    /// TODO: how to also correctly close websocket connections ?
    /// does tungstenite already take care of that when socket drop ?
    pub fn stop(self) {
        self.runtime.shutdown_timeout(Duration::from_secs(2));
    }
}
