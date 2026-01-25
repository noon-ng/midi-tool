use clap::{Parser, Subcommand};
use std::fmt::Display;

mod midi;
use midi::devices;

/// MIDI CLI tool
#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List available MIDI ports.
    List,

    /// Route MIDI messages.
    Route {
        /// Output port name
        #[arg(short, long)]
        target_name: String,

        /// Input port name
        #[arg(short, long)]
        source_name: String,

        /// Also print messages sent to the output.
        #[arg(short, long)]
        verbose: bool,
    },

    /// Monitor incoming MIDI messages.
    Monitor {
        /// Input port name
        #[arg(short, long)]
        source_name: String,
    },
}

#[derive(Debug)]
pub enum Errors {
    InitFailure,
    InvalidSourcePort(String),
    InvalidTargetPort(String),
    ForwardingError(String),
}

impl Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::InitFailure => write!(f, "Failed to initialize MIDI devices"),
            Errors::InvalidSourcePort(port) => write!(f, "Invalid source port: {}", port),
            Errors::InvalidTargetPort(port) => write!(f, "Invalid target port: {}", port),
            Errors::ForwardingError(message) => {
                write!(f, "Failed to forward MIDI messages: {}", message)
            }
        }
    }
}

fn main() -> std::process::ExitCode {
    let args = Args::parse();

    if let Err(e) = match args.command {
        Commands::List => devices::print(),
        Commands::Route {
            source_name,
            target_name,
            verbose,
        } => devices::route(source_name, target_name, verbose),
        Commands::Monitor { source } => devices::monitor(source),
    } {
        eprintln!("{}", e);
        return std::process::ExitCode::FAILURE;
    }

    std::process::ExitCode::SUCCESS
}
