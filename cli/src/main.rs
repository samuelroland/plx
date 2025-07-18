use clap::{Parser, Subcommand};
use parse::parse_command;
use plx_core::live::server::{DEFAULT_LIVE_PORT, LiveServer};
use simplelog::*;
use std::{fs::File, path::PathBuf, process::exit};

mod parse;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Starts the live server
    Server,
    /// Parse the given file or folder or the entire course with the PLX DY spec format
    Parse {
        #[arg(value_name = "PATH")]
        /// A PLX file with a .dy extension, or a folder with a course.dy
        path: PathBuf,
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
    let result = match cli.command {
        Commands::Server => {
            let server = LiveServer::new().expect("Couldn't setup LiveServer...");
            println!("Started PLX server on port {DEFAULT_LIVE_PORT}");
            server.start(DEFAULT_LIVE_PORT, true); // this is blocking indefinitly
            Ok(())
        }
        Commands::Parse { path } => parse_command(path),
    };

    if let Err(e) = result {
        eprintln!("Fatal error: {e}");
        exit(2);
    }
}
