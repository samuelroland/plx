use clap::{Parser, Subcommand};
use plx_core::live::server::{DEFAULT_LIVE_PORT, LiveServer};
use simplelog::*;
use std::{fs::File, path::PathBuf};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts the live server
    Server,
    /// Parse the given files or the entire course in the current folder otherwise
    Parse {
        #[arg(value_name = "FILE")]
        /// The list of DY files to parse with the .dy extension
        files: Option<Vec<PathBuf>>,
    },
}

fn main() {
    WriteLogger::init(
        LevelFilter::Debug,
        Config::default(),
        File::create("debug.log").expect("Failed to create log file"),
    )
    .expect("Failed to initialize WriteLogger");

    let cli = Cli::parse();

    // Start the server or the UI
    match cli.command {
        Some(Commands::Server) => {
            let server = LiveServer::new().expect("Couldn't setup LiveServer...");
            println!("Started PLX server on port {DEFAULT_LIVE_PORT}");
            server.start(DEFAULT_LIVE_PORT, true); // this is blocking indefinitly
        }
        Some(Commands::Parse { files }) => {
            dbg!(files);
        }
        None => {
            todo!()
        }
    }
}
