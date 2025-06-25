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

use crate::live::msg::Msg;

use futures_util::{stream::StreamExt, SinkExt};
use tokio::net::TcpListener;
use tokio::select;
use tokio_tungstenite::tungstenite::{
    client,
    handshake::{self, server::Callback},
    http::Response,
    Message, Utf8Bytes, WebSocket,
};

use super::{client::ClientRole, client_manager::ClientManager};

/// Version of the protocol, defined its specification
pub const PROTOCOL_VERSION: &str = "0.1.0";
/// Default port of the live protocol
pub const DEFAULT_LIVE_PORT: u16 = 9120;
/// Header sent during WebSocket handshake to announce the protocol version
const HEADER_LIVE_PROTOCOL_VERSION: &str = "LiveProtocolVersion";
/// Header sent during WebSocket handshake to announce the client id
const HEADER_LIVE_CLIENT_ID: &str = "LiveClientId";

/// The live server serving live sessions, this server is an async implementation
/// with the Tokio runtime
pub struct LiveServer {
    runtime: Runtime,
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
        let mut client_id = String::default(); // this is filled during check_handshake_callback

        let error_reponse = |body: String| {
            Response::builder()
                .status(400)
                .body(Some(body))
                .unwrap_or_default()
        };

        let check_handshake_callback = |request: &handshake::server::Request,
                                        response: handshake::server::Response|
         -> Result<
            handshake::server::Response,
            handshake::server::ErrorResponse,
        > {
            let protocol_version = request
                .headers()
                .get(HEADER_LIVE_PROTOCOL_VERSION)
                .ok_or_else(|| {
                    error_reponse(format!(
                        "Missing field {} in the HTTP headers.",
                        HEADER_LIVE_PROTOCOL_VERSION,
                    ))
                })?;

            let version = protocol_version
                .to_str()
                .map_err(|e| error_reponse(e.to_string()))?;

            if version != PROTOCOL_VERSION {
                return Err(error_reponse(format!("The server is only working with a live protocol version of {}, please update the client to match this version.", PROTOCOL_VERSION)));
            }

            client_id = request
                .headers()
                .get(HEADER_LIVE_CLIENT_ID)
                .ok_or_else(|| {
                    error_reponse(format!(
                        "Missing field {} in the HTTP headers.",
                        HEADER_LIVE_CLIENT_ID,
                    ))
                })?
                .to_str()
                .map_err(|e| error_reponse(e.to_string()))?
                .to_string();

            if client_id.trim().is_empty() {
                return Err(error_reponse(format!(
                    "Field {} is empty.",
                    HEADER_LIVE_CLIENT_ID
                )));
            }

            // let error = client_id = request.headers().get(HEADER_LIVE_CLIENT_ID)?;
            Ok(response)
        };

        // Accept the websocket stream with websocket handshake
        match tokio_tungstenite::accept_hdr_async(stream, check_handshake_callback).await {
            Ok(mut websocket) => {
                if client_id.is_empty() {
                    error!("The client_id cannot be empty at this point, it should have been checked before");
                    let _ = websocket.close(None).await;
                    return;
                }
                let mut client_manager = ClientManager {
                    client_id,
                    role: ClientRole::Follower,
                    websocket,
                    session: None,
                };

                // Let the client continue in its own separated task
                tokio::spawn(async move {
                    client_manager.run().await;
                });
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
