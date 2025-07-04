use chrono::{TimeDelta, Utc};
use core::panic;
use pretty_assertions::assert_eq;
use std::{
    sync::{mpsc::channel, Arc},
    thread::{self, sleep},
    time::Duration,
    vec,
};

use plx::live::{
    client::LiveClient,
    msg::{ClientNum, Event, ForwardedFile, LiveProtocolError},
    server::{LiveServer, PROTOCOL_VERSION},
    session::Session,
};

// Used most of the time as session name and group_id
const NAME: &str = "PRG2";
const GROUP_ID: &str = "PRG2group";
fn get_default_session() -> Session {
    Session {
        name: NAME.to_string(),
        group_id: GROUP_ID.to_string(),
    }
}

fn spawn_test_server() -> u16 {
    let random_dynamic_port = rand::random_range(49152..65535);
    // https://superuser.com/questions/956226/what-are-the-differences-between-the-3-port-types

    thread::spawn(move || {
        let server = LiveServer::new().unwrap();
        server.start(random_dynamic_port, false);
    });
    // just a short sleep so the server has time to start before clients start connecting
    thread::sleep(Duration::from_millis(100));
    random_dynamic_port
}

/// Spawn a server and N connected clients (no session yet)
fn spawn_server_and_n_clients(n: u16) -> Vec<LiveClient> {
    assert!(n > 0);
    let random_port = spawn_test_server();
    let mut clients = Vec::new();
    for i in 0..n {
        let c = LiveClient::connect("127.0.0.1", random_port, format!("SecretId{i}")).unwrap();
        clients.push(c);
    }
    clients
}

/// Create a session in addition to n clients, including the first client as the leader, the
/// following as followers of the session
fn spawn_server_and_n_clients_with_session(n: u16) -> Vec<Arc<LiveClient>> {
    assert!(n > 0);
    let mut clients = spawn_server_and_n_clients(n);
    let leader = &mut clients[0];
    leader.start_session(NAME, GROUP_ID).unwrap();
    for follower in clients[1..].iter_mut() {
        follower.join_session(NAME, GROUP_ID).unwrap();
    }

    clients.into_iter().map(Arc::new).collect()
}

// Tests around websocket connection
#[test]
#[ntest::timeout(2000)]
fn websocket_can_connect_with_client_id_and_protocol_version() {
    let port = spawn_test_server();
    let uuid = "19eff916-bca8-4b79-a94a-2342a3f2647d";
    let result = tokio_tungstenite::tungstenite::connect(format!(
        "ws://127.0.0.1:{port}/?live_client_id={uuid}&live_protocol_version={PROTOCOL_VERSION}"
    ));
    assert!(result.is_ok(), "failed with body {result:?}")
}
#[test]
#[ntest::timeout(2000)]
fn websocket_can_connect_with_client_id_containins_special_chars() {
    let port = spawn_test_server();
    let client_id = "what's that $??\"&\"*%ç*".to_string();
    let result = LiveClient::connect("localhost", port, client_id);
    assert!(result.is_ok())
}

#[test]
#[ntest::timeout(2000)]
fn websocket_fails_to_connect_without_info() {
    let port = spawn_test_server();
    let result = tokio_tungstenite::tungstenite::connect(format!("ws://127.0.0.1:{port}"));
    assert!(result.is_err(), "should have failed to connect {result:?}");
    // We cannot access the response body so I can only test the status code
    assert!(
        result
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("400 Bad Request"),
        "{}",
        result.unwrap_err().to_string()
    );
}

#[test]
#[ntest::timeout(2000)]
fn websocket_fails_with_missing_client_id() {
    let port = spawn_test_server();
    let result = tokio_tungstenite::tungstenite::connect(format!(
        "ws://127.0.0.1:{port}/?live_client_id=&live_protocol_version={PROTOCOL_VERSION}"
    ));
    assert!(result.is_err(), "should have failed to connect {result:?}");
    let result = tokio_tungstenite::tungstenite::connect(format!(
        "ws://127.0.0.1:{port}/?live_protocol_version={PROTOCOL_VERSION}"
    ));
    // We cannot access the response body so I can only test the status code
    assert!(
        result
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("400 Bad Request"),
        "{}",
        result.unwrap_err().to_string()
    );
}

#[test]
#[ntest::timeout(2000)]
fn websocket_fails_with_different_version_number() {
    let port = spawn_test_server();
    let result = tokio_tungstenite::tungstenite::connect(format!(
        "ws://127.0.0.1:{port}/?live_client_id=random-string&live_protocol_version=23.64.7"
    ));
    assert!(result.is_err(), "should have failed to connect {result:?}");
    // We cannot access the response body so I can only test the status code
    assert!(
        result
            .as_ref()
            .unwrap_err()
            .to_string()
            .contains("400 Bad Request"),
        "{}",
        result.unwrap_err().to_string()
    );
}

// Tests around session management

#[test]
#[ntest::timeout(4000)]
fn get_sessions_works() {
    let c = &mut spawn_server_and_n_clients(1)[0];
    assert_eq!(c.get_sessions("PRG2group".to_string()).unwrap(), vec![]);
    let expected_session = get_default_session();
    c.start_session(NAME, GROUP_ID).unwrap();
    assert_eq!(
        c.get_sessions(GROUP_ID.to_string()).unwrap(),
        vec![expected_session]
    );
}

#[test]
#[ntest::timeout(4000)]
fn get_sessions_correctly_use_group_id() {
    let c = &mut spawn_server_and_n_clients(5);
    println!("ok");
    c[0].start_session(NAME, GROUP_ID).unwrap();
    c[2].start_session("PRG1 Joe", "PRG1group").unwrap();
    c[3].start_session("PRG1 Joe", "PRG1FORK").unwrap();
    c[1].start_session("PRG1 Alice", "PRG1group").unwrap();

    assert_eq!(
        c[3].get_sessions("inexistant group id".to_string())
            .unwrap(),
        vec![]
    );
    assert_eq!(
        c[3].get_sessions("PRG2group".to_string()).unwrap(),
        vec![get_default_session()]
    );
    assert_eq!(
        c[3].get_sessions("PRG2group".to_string()).unwrap(),
        c[4].get_sessions("PRG2group".to_string()).unwrap(),
    );
    assert_eq!(
        c[3].get_sessions("PRG1group".to_string()).unwrap(),
        vec![
            // It also make sure the list is sorted !
            Session {
                name: "PRG1 Alice".to_string(),
                group_id: "PRG1group".to_string()
            },
            Session {
                name: "PRG1 Joe".to_string(),
                group_id: "PRG1group".to_string()
            },
        ]
    );
}

#[test]
#[ntest::timeout(4000)]
fn session_continues_to_exist_when_leader_disconnects() {
    let random_port = spawn_test_server();
    // Note: do not refactor with spawn_server_and_n_clients, we need to move out a client from vec in disconnect()
    let mut c0 = LiveClient::connect("127.0.0.1", random_port, "SecretId3".to_string()).unwrap();
    let mut c1 = LiveClient::connect("127.0.0.1", random_port, "SecretId4".to_string()).unwrap();
    c0.start_session(NAME, GROUP_ID).unwrap();
    assert_eq!(
        c1.get_sessions("PRG2group".to_string()).unwrap(),
        vec![get_default_session()]
    );
    c0.disconnect();
    sleep(Duration::from_millis(500));
    assert_eq!(
        c1.get_sessions("PRG2group".to_string()).unwrap(),
        vec![get_default_session()]
    );
    // The leader is back with same client_id !
    let mut c0 = LiveClient::connect("127.0.0.1", random_port, "SecretId3".to_string()).unwrap();
    c0.join_session(NAME, GROUP_ID).unwrap(); // this test that joining again make it a leader again
    c0.stop_session(); // if stopping the session works, it was a leader again
    c0.wait_on_next_event(); // some SessionStats
    let event = c0.wait_on_next_event();
    if let Some(Event::SessionStopped) = event {
    } else {
        panic!("{:?}", event)
    }

    assert_eq!(c1.get_sessions("PRG2group".to_string()).unwrap(), vec![]);
}

#[test]
#[ntest::timeout(4000)]
fn exo_switch_from_leader_is_forwarded_when_session_exists() {
    let random_port = spawn_test_server();
    let mut c = LiveClient::connect("127.0.0.1", random_port, "SecretId3".to_string()).unwrap();
    c.start_session(NAME, GROUP_ID).unwrap();
    c.send_exo_switch("intro/salue-moi".to_string());

    let (tx, rx) = channel::<Event>();
    thread::spawn(move || {
        c.wait_all_next_events(tx);
        // while let Ok(a) = rx.recv() {
        //     // emit tauri event
        //     println!("{a:?}");
        // }
    });
    assert_eq!(
        rx.recv().unwrap(),
        Event::ExoSwitched {
            path: "intro/salue-moi".to_string()
        }
    );
}

#[test]
#[ntest::timeout(4000)]
fn exo_switch_from_follower_fails() {
    let mut c = spawn_server_and_n_clients(2);
    c[0].start_session(NAME, GROUP_ID).unwrap();
    c[1].join_session(NAME, GROUP_ID).unwrap();

    c[1].send_exo_switch("intro/salue-moi".to_string());

    let res = c[1].wait_on_next_event().unwrap();
    if let Event::Error(LiveProtocolError::ActionOnlyForLeader(_)) = res {
    } else {
        panic!("Expected SessionNotFound error, got {:?}", res);
    }
}

#[test]
#[ntest::timeout(2000)]
fn exo_switch_without_session_fails() {
    let c = &mut spawn_server_and_n_clients(1)[0];
    c.send_exo_switch("intro/salue-moi".to_string());

    let res = c.wait_on_next_event().unwrap();
    if let Event::Error(LiveProtocolError::SessionNotFound) = res {
    } else {
        panic!("Expected SessionNotFound error, got {:?}", res);
    }
}

/// Make sure 2 events are equal or panic
/// Consider a time variation of 2 secondes to be equal timestamp
fn assert_events_eq(e1: &Event, e2: &Event) {
    match e1 {
        Event::ForwardFile { client_num, file } => {
            if let Event::ForwardFile {
                client_num: client_num2,
                file: file2,
            } = e2
            {
                assert_eq!(client_num, client_num2);
                assert_eq!(file.path, file2.path);
                assert_eq!(file.content, file2.content);
                assert!((file.time - file2.time).abs() < TimeDelta::seconds(2));
                return;
            }
            panic!("{e1:?} and {e2:?} should have equal type !");
        }
        Event::ForwardResult { client_num, result } => {
            if let Event::ForwardResult {
                client_num: client_num2,
                result: result2,
            } = e2
            {
                assert_eq!(client_num, client_num2);
                assert_eq!(result.check_result, result2.check_result);

                assert!((result.time - result2.time).abs() < TimeDelta::seconds(2));
                return;
            }
            panic!("{e1:?} and {e2:?} should have equal type !");
        }
        _ => assert_eq!(e1, e2),
    }
}

#[test]
#[ntest::timeout(2000)]
fn forwarding_to_leaders_work() {
    let random_port = spawn_test_server();
    let mut c0 = LiveClient::connect("127.0.0.1", random_port, format!("SecretId{}", 1)).unwrap();
    let mut c1 = LiveClient::connect("127.0.0.1", random_port, format!("SecretId{}", 2)).unwrap();
    let mut c2 = LiveClient::connect("127.0.0.1", random_port, format!("SecretId{}", 3)).unwrap();
    c0.start_session(NAME, GROUP_ID).unwrap();
    c1.join_session(NAME, GROUP_ID).unwrap();
    c2.join_session(NAME, GROUP_ID).unwrap();
    c0.wait_on_next_event().unwrap(); // consume the 2 stats
    c0.wait_on_next_event().unwrap(); // consume the 2 stats

    c1.send_file("main.c".to_string(), "client 1, code v1".to_string());
    sleep(Duration::from_millis(200));
    c2.send_file("main.c".to_string(), "client 2, code v1".to_string());
    sleep(Duration::from_millis(200));
    c1.send_file("main.c".to_string(), "client 1, code v2".to_string());
    let now = Utc::now();

    assert_events_eq(
        &c0.wait_on_next_event().unwrap(),
        &Event::ForwardFile {
            client_num: ClientNum(2),
            file: ForwardedFile {
                path: "main.c".to_string(),
                content: "client 1, code v1".to_string(),
                time: now,
            },
        },
    );

    assert_events_eq(
        &c0.wait_on_next_event().unwrap(),
        &Event::ForwardFile {
            client_num: ClientNum(3),
            file: ForwardedFile {
                path: "main.c".to_string(),
                content: "client 2, code v1".to_string(),
                time: now,
            },
        },
    );

    assert_events_eq(
        &c0.wait_on_next_event().unwrap(),
        &Event::ForwardFile {
            client_num: ClientNum(2),
            file: ForwardedFile {
                path: "main.c".to_string(),
                content: "client 1, code v2".to_string(),
                time: now,
            },
        },
    );
}
