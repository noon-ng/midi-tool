use clap::{Parser, Subcommand};
use midir::{MidiInput, MidiOutput};
use std::fmt::Display;

mod midi;
use midi::{Devices, Route};

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
    },
}

#[derive(Debug)]
enum Errors {
    InitFailure,
    InvalidInputPort(String),
    InvalidOutputPort(String),
    ForwardingError,
}

impl Display for Errors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Errors::InitFailure => write!(f, "Failed to initialize MIDI devices"),
            Errors::InvalidInputPort(port) => write!(f, "Invalid input port: {}", port),
            Errors::InvalidOutputPort(port) => write!(f, "Invalid output port: {}", port),
            Errors::ForwardingError => write!(f, "Failed to forward MIDI messages"),
        }
    }
}

fn main() -> Result<(), Errors> {
    let args = Args::parse();

    let devices = Devices::new(
        MidiInput::new("MIDI Input").map_err(|_| Errors::InitFailure)?,
        MidiOutput::new("MIDI Output").map_err(|_| Errors::InitFailure)?,
    );

    match args.command {
        Commands::List => devices.print(),
        Commands::Route {
            source_name,
            target_name,
        } => {
            let Some(target) = devices.find_output_port(&target_name) else {
                return Err(Errors::InvalidOutputPort(target_name.to_string()));
            };

            let Some(source) = devices.find_input_port(&source_name) else {
                return Err(Errors::InvalidInputPort(source_name.to_string()));
            };

            let route = Route::new(source, target);

            devices
                .activate(route)
                .map_err(|_| Errors::ForwardingError)?;
        }
    }

    Ok(())
}
