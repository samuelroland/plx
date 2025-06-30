use pretty_assertions::assert_eq;
use std::{
    sync::mpsc::channel,
    thread::{self, sleep},
    time::Duration,
    vec,
};

use plx::live::{
    client::{LiveClient, ProtocolError},
    msg::{Event, LiveProtocolError},
    server::LiveServer,
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
        let c = LiveClient::connect("127.0.0.1", random_port, format!("SecretId{}", i)).unwrap();
        clients.push(c);
    }
    clients
}

/// Create a session in addition to n clients, including the first client as the leader, the
/// following as followers of the session
fn spawn_server_and_n_clients_with_session(n: u16) -> Vec<LiveClient> {
    assert!(n > 0);
    let mut clients = spawn_server_and_n_clients(n);
    let mut it = clients.iter_mut();
    let leader: &mut LiveClient = it.next().unwrap();
    leader.start_session(NAME, GROUP_ID).unwrap();
    for follower in it {
        follower.join_session(NAME, GROUP_ID).unwrap();
    }
    clients
}

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
    c0.stop_session().unwrap(); // if stopping the session works, it was a leader again
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
        c.training_events_subscribe(tx);
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
    let (tx, rx) = channel::<Event>();
    thread::spawn(move || {
        c[1].training_events_subscribe(tx);
        // while let Ok(a) = rx.recv() {
        //     // emit tauri event
        //     println!("{a:?}");
        // }
    });
    assert_eq!(
        rx.recv().unwrap(),
        Event::Error(LiveProtocolError::ActionOnlyForLeader(
            "Switch exo".to_string()
        )) // Event::ExoSwitched {
           //     path: "intro/salue-moi".to_string()
           // }
    );
    //
    // if let Err() = result {
    // } else {
    //     panic!("Expected ActionOnlyForLeader error, got {:?}", result);
    // }
}

#[test]
#[ntest::timeout(2000)]
fn exo_switch_without_session_fails() {
    let c = &mut spawn_server_and_n_clients(1)[0];
    let result = c.send_exo_switch("intro/salue-moi".to_string());
    //
    // if let Err(ProtocolError::Live(LiveProtocolError::SessionNotFound)) = result {
    // } else {
    //     panic!("Expected SessionNotFound error, got {:?}", result);
    // }
}
