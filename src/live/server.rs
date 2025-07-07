use std::{sync::Arc, time::Duration};

use log::{error, warn};
use tokio::{
    net::TcpStream,
    runtime::Runtime,
    sync::mpsc::{self},
};

use tokio::net::TcpListener;
use tokio::select;
use tokio_tungstenite::tungstenite::{handshake, http::Response};
use url::Url;

use super::{client_manager::ClientManager, sessions_manager::SessionsManager};

/// Version of the protocol, defined its specification
pub const PROTOCOL_VERSION: &str = "0.1.0";
/// Default port of the live protocol
pub const DEFAULT_LIVE_PORT: u16 = 9120;
/// Query string field sent during WebSocket handshake to announce the protocol version
pub const QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD: &str = "live_protocol_version";
/// Query string field sent during WebSocket handshake to announce the client id
pub const QUERYSTRING_LIVE_CLIENT_ID_FIELD: &str = "live_client_id";

/// The live server serving live sessions, this server is an async implementation
/// with the Tokio runtime
pub struct LiveServer {
    runtime: Runtime,
    sessions_manager: Arc<SessionsManager>,
}

impl LiveServer {
    /// Create a live server and prepare a Tokio runtime for
    pub fn new() -> Result<Self, std::io::Error> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()?;
        let sessions_manager = Arc::new(SessionsManager::new());
        Ok(LiveServer {
            runtime,
            sessions_manager,
        })
    }

    /// Start a server listening on given port, use the DEFAULT_LIVE_PORT or a custom one
    /// listening on all network interfaces ("0.0.0.0") to be publicly accessible
    /// This function is blocking and will never stop, until there is a SIGINT signal and the
    /// shutdown is managed properly to close all connections and shutdown the runtime before return
    pub fn start(self, port: u16, handle_clean_shutdown: bool) {
        // TODO: make sure we cannot start twice !
        self.runtime.block_on(async {
            // Graceful shutdown management, with a first async channel to receive another sync
            // chanel to send the event to indicate "that's down all good"
            let (shutdown_tx, mut shutdown_rx) = mpsc::unbounded_channel::<()>();
            if handle_clean_shutdown {
                // Listen on SIGINT signal and wait for the confirmation of shutdown
                // Disable during testing
                ctrlc::set_handler(move || {
                    println!("\nDetected shutdown signal, starting shutdown process...");
                    let _ = shutdown_tx.send(());
                })
                .expect("Error setting Ctrl-C handler");
            }

            // Start binding here, so it can fail if the port is already used.
            let listener = TcpListener::bind(format!("0.0.0.0:{port}"))
                .await
                .expect("Couldn't not bind on port {port}");

            // On all new TCP connections, just spawn a new task to process the new client
            loop {
                select! {
                    new_client = listener.accept() => {
                        if let Ok((stream, _)) = new_client {
                            tokio::spawn(Self::process_client(
                                stream,
                                Arc::clone(&self.sessions_manager),
                            ));
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        println!("SessionsManager call of shutdown process");
                        self.sessions_manager.shutdown().await;
                        println!("SessionsManager is done");
                        break; // so the final shutdown of the runtime can be done
                    }
                };
            }
        });

        println!("Shutting down the Tokio runtime");
        self.runtime.shutdown_timeout(Duration::from_secs(2));
    }

    /// Once a TcpSocket has been accepted into a TcpStream, we can start the websocket connection
    async fn process_client(stream: TcpStream, sessions_manager: Arc<SessionsManager>) {
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
            // Note: we have no way to access the full URL, we need to prepend it with what's after
            // the path, it already include the first /
            let url = Url::parse(&format!("ws://localhost{}", request.uri()))
                .map_err(|e| error_reponse(e.to_string()))?;
            let mut protocol_version = String::default();
            for (key, value) in url.query_pairs() {
                if key == QUERYSTRING_LIVE_CLIENT_ID_FIELD {
                    client_id = value.to_string();
                    continue;
                }

                if key == QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD {
                    protocol_version = value.to_string();
                    continue;
                }
            }

            if protocol_version.is_empty() {
                return Err(error_reponse(format!(
                    "Missing field {QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD} in the query string",
                )));
            }

            if protocol_version != PROTOCOL_VERSION {
                return Err(error_reponse(format!("The server is only working with a live protocol version of {PROTOCOL_VERSION}, please update the client to match this version.")));
            }

            if client_id.is_empty() {
                return Err(error_reponse(format!(
                    "Missing field {QUERYSTRING_LIVE_CLIENT_ID_FIELD} in the query string",
                )));
            }

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
                    websocket,
                    session: None,
                    sessions_manager,
                };

                // Let the client continue in its own separated task
                tokio::spawn(async move {
                    client_manager.run().await;
                });
            }
            Err(e) => warn!("Got a handshake error: {e}"),
        }
    }
}
