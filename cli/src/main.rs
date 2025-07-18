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
    /// Parse the given DY file or parse the course.dy inside given folder
    Parse {
        #[arg(value_name = "PATH")]
        /// A PLX file with a .dy extension, or a folder with a course.dy
        path: PathBuf,
        #[arg(long, default_value_t = false)]
        /// Enable the full course parsing in PLX's format.
        /// Only valid with a folder
        full: bool,
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
        Commands::Parse { path, full } => parse_command(path, full),
    };

    if let Err(e) = result {
        eprintln!("Fatal error: {e}");
        exit(2);
    }
}
