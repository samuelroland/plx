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
    handshake::{self, server::Callback},
    http::Response,
    Message, Utf8Bytes, WebSocket,
};

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
        let mut client_id: Option<String> = None;
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
                    Response::builder()
                        .status(400)
                        .body(Some(
                            "Missing LiveProtocolVersion field in the HTTP headers.".to_string(),
                        ))
                        .unwrap_or_default()
                })?;

            let version = protocol_version.to_str().map_err(|e| {
                Response::builder()
                    .status(400)
                    .body(Some(format!("{}", e)))
                    .unwrap_or_default()
            })?;

            if version != PROTOCOL_VERSION {
                return Err(Response::builder()
                        .status(400)
                        .body(Some(format!("The server is only working with a live protocol version of {}, please make sure the client match this need.", PROTOCOL_VERSION)))
                        .unwrap_or_default());
            }

            // let error = client_id = request.headers().get(HEADER_LIVE_CLIENT_ID)?;
            Ok(response)
        };

        // Accept the websocket stream with websocket handshake
        match tokio_tungstenite::accept_hdr_async(stream, check_handshake_callback).await {
            Ok(mut websocket) => {
                let client = Client {
                    client_id,
                    role: ClientRole::Follower,
                    websocket,
                    external_write_rx: todo!(),
                    session_tx: todo!(),
                    server_tx: todo!(),
                };

                // Let the client continue in its own separated task
                tokio::spawn(client.run());
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
