use std::future::IntoFuture;
/// Client implementation of the live protocol
use std::{fmt::Display, sync::mpsc::Sender, thread, time::Duration};

use std::net::TcpStream;

use futures_util::SinkExt;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{tungstenite::stream::MaybeTlsStream, WebSocketStream};
use url::Url;

use super::{
    msg::{Action, Event},
    server::{
        PROTOCOL_VERSION, QUERYSTRING_LIVE_CLIENT_ID_FIELD, QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD,
    },
};

use tokio_tungstenite::tungstenite;

use super::{
    msg::{ClientNum, ExoCheckResult, LiveProtocolError},
    session::Session,
};

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub struct LiveClient {
    /// A transmitter where we can send Action for the server
    pub(super) send: UnboundedSender<Action>,
    /// All events bac
    pub(super) recv: UnboundedReceiver<Event>,

    client_num: Option<ClientNum>,
}

#[derive(Debug)]
pub enum ProtocolError {
    Live(LiveProtocolError),
    Network(Box<tungstenite::Error>),
    UnexpectedMsg(String),
}

impl Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            match self {
                ProtocolError::Live(live_protocol_error) => {
                    format!("Live protocol error: {live_protocol_error}")
                }
                ProtocolError::Network(error) => format!("Network error: {error}"),
                ProtocolError::UnexpectedMsg(text) => {
                    format!("Unexpected message received from the server: {text}")
                }
            }
            .as_ref(),
        )
    }
}

impl LiveClient {
    pub fn connect(
        domain: &str,
        port: u16,
        client_id: String,
    ) -> Result<LiveClient, std::io::Error> {
        let domain = domain.to_string();

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();

        let (send_tx, mut send_rx) = unbounded_channel::<Action>();
        let (recv_tx, recv_rx) = unbounded_channel::<Event>();
        // Just prepare the future to run on the runtime
        let runtime_handle = runtime.spawn(async move {
            println!("Starting ClientSplitter tokio runtime");
            let link = format!("ws://{domain}:{port}");
            let url = Url::parse_with_params(&link, &[(QUERYSTRING_LIVE_CLIENT_ID_FIELD, client_id.as_ref()), (QUERYSTRING_LIVE_PROTOCOL_VERSION_FIELD, PROTOCOL_VERSION)]).unwrap();
            let (mut socket, _) = tokio_tungstenite::connect_async(url.to_string()).await.unwrap();
            loop {
                select! {
                    // Read messages from socket and forward them
                    ws_msg = socket.next() => {
                        if let Some(Ok(ws_msg)) = ws_msg {
                            println!("LiveClient: got {ws_msg:?}");
                            match ws_msg.into_text().ok().and_then(|txt| Event::try_from(txt).ok()) {
                                    Some(event) => {
                                        let _ = recv_tx.send(event.clone());
                                    }
                                    None => {
                                        eprintln!("Failed to parse event from ws_msg");
                                        continue;
                                    }
                                }
                        }
                    }
                    // Read actions to sent into socket
                    action = send_rx.recv() => {
                        if let Some(action) = action {
                            let msg = action.try_into();
                            if let Ok(msg) = msg {
                                let _ = socket.send(Message::Text(msg)).await;
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        });
        // Start with the first future, but block on another thread so we can return here
        thread::spawn(move || {
            runtime.block_on(runtime_handle.into_future()).unwrap();
            runtime.shutdown_timeout(Duration::from_secs(2));
        });

        // TODO: okay ??
        thread::sleep(Duration::from_millis(100));

        let client = LiveClient {
            send: send_tx,
            recv: recv_rx,
            client_num: None,
        };

        Ok(client)
    }

    pub fn disconnect(self) {
        // just do nothing, let the struct and self.mp drop to close the self.mp.send
        // on the other end the receiver.recv() will return none which should stop the Splitter
        // and at the same time close the websocket on drop
    }

    /// Just sending a Msg on the socket
    pub fn send_msg(&mut self, action: Action) {
        println!("Sending: {action:?}");
        self.send.send(action).unwrap();
    }

    pub fn wait_on_next_event(&mut self) -> Option<Event> {
        self.recv.blocking_recv()
    }

    pub fn wait_all_next_events(&mut self, tx: Sender<Event>) {
        while let Some(a) = self.wait_on_next_event() {
            println!("{a:?}");
            tx.send(a).unwrap();
        }
    }

    /// Create a new session
    pub fn start_session(&mut self, name: &str, group_id: &str) -> Result<Session, String> {
        self.send_msg(Action::StartSession {
            name: name.to_string(),
            group_id: group_id.to_string(),
        });
        let event = self.wait_on_next_event();
        if let Some(Event::SessionJoined(client_num)) = event {
            self.client_num = Some(client_num);
            Ok(Session {
                name: name.to_string(),
                group_id: group_id.to_string(),
            })
        } else {
            Err(format!("{:?}", event))
        }
    }

    /// Create a new session
    pub fn stop_session(&mut self) {
        self.send_msg(Action::StopSession);
    }

    /// Join a session
    pub fn join_session(&mut self, name: &str, group_id: &str) -> Result<Session, String> {
        self.send_msg(Action::JoinSession {
            name: name.to_string(),
            group_id: group_id.to_string(),
        });
        if let Some(Event::SessionJoined(client_num)) = self.wait_on_next_event() {
            println!("Joined session '{name}'");
            self.client_num = Some(client_num)
        }
        Ok(Session {
            name: name.to_string(),
            group_id: group_id.to_string(),
        })
    }

    /// Non blocking way to leave the session
    pub fn leave_session(&mut self) {
        self.send_msg(Action::LeaveSession);
    }

    /// Send a file content after a change
    pub fn send_file(&mut self, file: String, content: String) {
        self.send_msg(Action::SendFile {
            path: file,
            content,
        });
    }

    /// Send a check result
    pub fn send_result(&mut self, check_result: ExoCheckResult) {
        self.send_msg(Action::SendResult { check_result });
    }

    /// Send a check result
    pub fn send_exo_switch(&mut self, path: String) {
        self.send_msg(Action::SwitchExo { path });
    }

    /// Get all available session for a given group id
    pub fn get_sessions(&mut self, group_id: String) -> Result<Vec<Session>, String> {
        self.send_msg(Action::GetSessions { group_id });
        let e = self.wait_on_next_event();
        if let Some(Event::SessionsList(list)) = e {
            return Ok(list);
        }
        Err("Couldn't get sessions list".to_string())
    }
}
