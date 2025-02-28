use midir::{Ignore, MidiInput, MidiInputPort, MidiOutput, MidiOutputPort};
use std::error::Error;
use std::io::stdin;

use crate::Route;

pub struct Devices {
    input: MidiInput,
    output: MidiOutput,
}

impl Devices {
    pub fn new(input: MidiInput, output: MidiOutput) -> Self {
        Self { input, output }
    }

    pub fn find_input_port(&self, port_name: &str) -> Option<MidiInputPort> {
        self.input_ports()
            .into_iter()
            .find(|port| self.input.port_name(port) == Ok(port_name.to_string()))
    }

    pub fn find_output_port(&self, port_name: &str) -> Option<MidiOutputPort> {
        self.output_ports()
            .into_iter()
            .find(|port| self.output.port_name(port) == Ok(port_name.to_string()))
    }

    pub fn print(&self) {
        match self.input_ports().len() {
            0 => println!("No input ports found."),
            _ => {
                println!("Input ports: ");

                self.input_ports().iter().enumerate().for_each(|(i, port)| {
                    println!("{}: {}", i, self.input.port_name(port).unwrap())
                });
            }
        }

        match self.output_ports().len() {
            0 => println!("No output ports found."),
            _ => {
                println!("Output ports: ");

                self.output_ports()
                    .iter()
                    .enumerate()
                    .for_each(|(i, port)| {
                        println!("{}: {}", i, self.output.port_name(port).unwrap())
                    });
            }
        }
    }

    fn input_ports(&self) -> Vec<MidiInputPort> {
        self.input.ports()
    }
    fn output_ports(&self) -> Vec<MidiOutputPort> {
        self.output.ports()
    }

    pub fn activate(mut self, route: Route) -> Result<(), Box<dyn Error>> {
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
}
