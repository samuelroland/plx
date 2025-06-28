use std::{thread, time::Duration};

use plx::live::{
    client::{LiveClient, ProtocolError},
    msg::LiveProtocolError,
    server::{LiveServer, DEFAULT_LIVE_PORT},
};
use rand::random;

fn spawn_test_server() -> u16 {
    let random_dynamic_port = rand::random_range(49152..65535);
    // https://superuser.com/questions/956226/what-are-the-differences-between-the-3-port-types

    thread::spawn(move || {
        let server = LiveServer::new().unwrap();
        server.start(random_dynamic_port);
    });
    thread::sleep(Duration::from_secs(1));
    random_dynamic_port
}

#[test]
#[ntest::timeout(4000)]
fn exo_switch_from_leader_is_forwarded_when_session_exists() {
    let random_port = spawn_test_server();
    let mut client = LiveClient::connect("127.0.0.1", random_port, "client 1".to_string()).unwrap();
    client
        .start_session("PRG2", "PRG2group".to_string())
        .unwrap();
    client
        .send_exo_switch("intro/salue-moi".to_string())
        .unwrap(); // unwrap is making sure we got an ExoSwitched back
}

#[test]
#[ntest::timeout(4000)]
fn exo_switch_from_follower_fails() {
    let random_port = spawn_test_server();
    let mut c = LiveClient::connect("127.0.0.1", random_port, "client 1".to_string()).unwrap();
    let mut c2 = LiveClient::connect("127.0.0.1", random_port, "client 2".to_string()).unwrap();
    c.start_session("PRG2", "PRG2group".to_string()).unwrap();
    c2.join_session("PRG2", "PRG2group".to_string()).unwrap();

    let result = c2.send_exo_switch("intro/salue-moi".to_string());

    if let Err(ProtocolError::Live(LiveProtocolError::ActionOnlyForLeader(a))) = result {
    } else {
        panic!("Expected ActionOnlyForLeader error, got {:?}", result);
    }
}

#[test]
#[ntest::timeout(4000)]
fn exo_switch_without_session_fails() {
    let random_port = spawn_test_server();
    let mut c = LiveClient::connect("127.0.0.1", random_port, "client 1".to_string()).unwrap();
    let result = c.send_exo_switch("intro/salue-moi".to_string());

    if let Err(ProtocolError::Live(LiveProtocolError::SessionNotFound)) = result {
    } else {
        panic!("Expected SessionNotFound error, got {:?}", result);
    }
}
