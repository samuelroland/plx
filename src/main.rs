use crate::app::app::App;
use live::server::{LiveServer, DEFAULT_LIVE_PORT};
use simplelog::*;
use std::{env::args, fs::File};
pub mod app;
pub mod core;
pub mod live;
pub mod models;

fn main() {
    WriteLogger::init(
        LevelFilter::Trace,
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
        match App::new() {
            Ok(app) => app.run_forever(),
            Err(err) => {
                eprintln!("Error starting plx {err}");
            }
        }
    }
}
