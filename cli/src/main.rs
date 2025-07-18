use plx_core::live::server::{DEFAULT_LIVE_PORT, LiveServer};
use simplelog::*;
use std::{env::args, fs::File};

fn main() {
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("debug.log").expect("Failed to create log file"),
    )
    .expect("Failed to initialize WriteLogger");
    // Start the server or the UI
    let args: Vec<String> = args().collect();
    if args.len() > 1 && args[1] == "server" {
        let server = LiveServer::new().expect("Couldn't setup LiveServer...");
        println!("Started PLX server on port {DEFAULT_LIVE_PORT}");
        server.start(DEFAULT_LIVE_PORT, true); // this is blocking indefinitly
    } else {
        eprintln!("Only the server is available here, use 'server' argument");
    }
}
