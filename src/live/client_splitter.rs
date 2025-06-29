/// Client implementation of the live protocol
use std::{net::TcpStream, time::Duration};

use futures_util::SinkExt;
use tokio::select;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_stream::StreamExt;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{
    tungstenite::{http::Uri, stream::MaybeTlsStream, ClientRequestBuilder},
    WebSocketStream,
};

use super::{
    msg::{Action, Event},
    server::{HEADER_LIVE_CLIENT_ID, HEADER_LIVE_PROTOCOL_VERSION, PROTOCOL_VERSION},
};

/// The 3 transmitters for the LiveClient
pub(super) struct SplitterInterface {
    /// A transmitter where we can send Action for the server
    pub(super) send: UnboundedSender<Action>,
    /// Where we receive session related events and error, any event we are specifically waiting on.
    /// For example, after StartSession, we expect and wait to receive a SessionStarted event.
    pub(super) session_recv: UnboundedReceiver<Event>,
    /// Where we receive events related to training that will be only streamed
    /// as we don't know which order or which request might
    /// For example, we want to receive in a streaming mode (via message passing) the
    /// ForwardResult, ForwardFile and ExoSwitch.
    pub(super) training_recv: UnboundedReceiver<Event>,
}

struct SplitterMp {
    send: UnboundedReceiver<Action>,
    session_recv: UnboundedSender<Event>,
    training_recv: UnboundedSender<Event>,
}

pub(super) struct ClientSplitter {
    mp: Option<SplitterMp>,
}

type Ws = WebSocketStream<MaybeTlsStream<TcpStream>>;

impl ClientSplitter {
    pub(super) fn new() -> (Self, SplitterInterface) {
        let (send_tx, send_rx) = unbounded_channel::<Action>();
        let (session_tx, session_rx) = unbounded_channel::<Event>();
        let (training_tx, training_rx) = unbounded_channel::<Event>();
        let interface = SplitterInterface {
            send: send_tx,
            session_recv: session_rx,
            training_recv: training_rx,
        };
        let mp = SplitterMp {
            send: send_rx,
            session_recv: session_tx,
            training_recv: training_tx,
        };
        (Self { mp: Some(mp) }, interface)
    }

    pub(super) fn start(&mut self, domain: &str, port: u16, client_id: String) {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_io()
            .build()
            .unwrap();
        runtime.block_on(async move {
            let uri: Uri = format!("ws://{}:{}", domain, port).parse().unwrap(); // todo fix unwrap + todo support TLS !
            let builder = ClientRequestBuilder::new(uri)
                .with_header(HEADER_LIVE_PROTOCOL_VERSION, PROTOCOL_VERSION)
                .with_header(HEADER_LIVE_CLIENT_ID, client_id);
            let (mut socket, _) = tokio_tungstenite::connect_async(builder).await.unwrap();
            let mut mp = self.mp.take().unwrap();
            tokio::spawn(async move {
                loop {
                    select! {
                        // Read messages from socket and forward them in the correct channel
                        ws_msg = socket.next() => {
                            if let Some(Ok(ws_msg)) = ws_msg {
                                let event: Event = Event::try_from(ws_msg.into_text().unwrap()).unwrap();
                                match event {
                                    Event::SessionStarted
                                    | Event::SessionStopped
                                    | Event::SessionJoined
                                    | Event::SessionsList(..)
                                    | Event::Error(..) => {
                                        let _ = mp.session_recv.send(event.clone());
                                    }
                                    Event::ExoSwitched { .. }  | Event::ForwardFile(..)  | Event::ForwardResult(..)  | Event::Stats(..) => {
                                        let _ = mp.training_recv.send(event.clone());
                                    }
                                    Event::ServerStopped => {
                                        mp.send.close();
                                        mp.session_recv.closed().await;
                                        mp.training_recv.closed().await;
                                        break;
                                    }
                                }
                                let _ = mp.session_recv.send(event);
                            } else {
                                break;
                            }
                        }
                        // Read actions to sent into socket
                        action = mp.send.recv() => {
                            if let Some(action) = action {
                                let _ = socket.send(Message::Text(action.try_into().unwrap())).await;
                            } else {
                                break;
                            }
                        }
                    }
                }
            });
        });
        runtime.shutdown_timeout(Duration::from_secs(2));
    }
}
