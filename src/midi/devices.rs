use midir::{Ignore, MidiInput, MidiInputPort, MidiOutput, MidiOutputPort};
use std::error::Error;
use std::io::stdin;

use crate::Errors;

pub struct Devices {
    input: MidiInput,
    output: MidiOutput,
}

struct Route {
    pub source: MidiInputPort,
    pub target: MidiOutputPort,
}

impl Devices {
    pub fn new() -> Result<Self, Errors> {
        match (MidiInput::new("MIDI Input"), MidiOutput::new("MIDI Output")) {
            (Ok(input), Ok(output)) => Ok(Self { input, output }),
            _ => Err(Errors::InitFailure),
        }
    }

    pub fn print(&self) {
        match self.input.ports().len() {
            0 => println!("No input ports found."),
            _ => {
                println!("Input ports: ");

                self.input.ports().iter().enumerate().for_each(|(i, port)| {
                    println!("{}: {}", i, self.input.port_name(port).unwrap())
                });
            }
        }

        match self.output.ports().len() {
            0 => println!("No output ports found."),
            _ => {
                println!("Output ports: ");

                self.output
                    .ports()
                    .iter()
                    .enumerate()
                    .for_each(|(i, port)| {
                        println!("{}: {}", i, self.output.port_name(port).unwrap())
                    });
            }
        }
    }

    pub fn route(self, source_name: String, target_name: String) -> Result<(), Errors> {
        let target = self
            .find_output_port(&target_name)
            .ok_or(Errors::InvalidOutputPort(target_name))?;
        let source = self
            .find_input_port(&source_name)
            .ok_or(Errors::InvalidInputPort(source_name))?;

        let route = Route { source, target };

        match self.activate(route) {
            Ok(_) => Ok(()),
            Err(_) => Err(Errors::ForwardingError),
        }
    }
    fn activate(mut self, route: Route) -> Result<(), Box<dyn Error>> {
        println!("Activating route:");
        println!("  Input port: {}", self.input.port_name(&route.source)?);
        println!("  Output port: {}", self.output.port_name(&route.target)?);

        let mut outgoing_connection = self.output.connect(&route.target, "midi-router")?;

        self.input.ignore(Ignore::None);

        // Debug prints in the callback
        let _connection = self.input.connect(
            &route.source,
            "midi-router",
            move |_stamp, message, _| {
                // Forward the message
                if let Err(e) = outgoing_connection.send(message) {
                    println!("Error sending message: {:?}", e)
                }
            },
            (),
        )?;

        stdin().read_line(&mut String::new())?;
        Ok(())
    }

    fn find_input_port(&self, port_name: &str) -> Option<MidiInputPort> {
        self.input
            .ports()
            .into_iter()
            .find(|port| self.input.port_name(port) == Ok(port_name.to_string()))
    }

    fn find_output_port(&self, port_name: &str) -> Option<MidiOutputPort> {
        self.output
            .ports()
            .into_iter()
            .find(|port| self.output.port_name(port) == Ok(port_name.to_string()))
    }
}
