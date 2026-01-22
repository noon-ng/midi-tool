use midir::{MidiInput, MidiInputPort, MidiOutput, MidiOutputPort};

use crate::Errors;
use crate::midi::route::Route;
use crate::midi::target::{MonitorTarget, OutputTarget, Target};

pub struct Devices {
    input: MidiInput,
}

impl Devices {
    pub fn new() -> Result<Self, Errors> {
        match MidiInput::new("MIDI Input") {
            Ok(input) => Ok(Self { input }),
            _ => Err(Errors::InitFailure),
        }
    }

    pub fn print(&self) {
        match self.input.ports().len() {
            0 => println!("No input ports found."),
            _ => {
                println!("Input ports: ");

                self.input.ports().iter().enumerate().for_each(|(i, port)| {
                    let name = self
                        .input
                        .port_name(port)
                        .unwrap_or_else(|_| "<unknown>".to_string());
                    println!("{}: {}", i, name)
                });
            }
        }

        match MidiOutput::new("MIDI Output") {
            Ok(output) => match output.ports().len() {
                0 => println!("No output ports found."),
                _ => {
                    println!("Output ports: ");

                    output.ports().iter().enumerate().for_each(|(i, port)| {
                        let name = output
                            .port_name(port)
                            .unwrap_or_else(|_| "<unknown>".to_string());
                        println!("{}: {}", i, name)
                    });
                }
            },
            _ => println!("Failed to initialize MIDI output."),
        }
    }

    pub fn route(
        self,
        source_name: String,
        target_name: String,
        verbose: bool,
    ) -> Result<(), Errors> {
        let source = self
            .find_input_port(&source_name)
            .ok_or(Errors::InvalidInputPort(source_name))?;

        let output = MidiOutput::new("MIDI Output").map_err(|_| Errors::InitFailure)?;
        let target = Self::find_output_port(&output, &target_name)
            .ok_or(Errors::InvalidOutputPort(target_name))?;

        let mut targets: Vec<Box<dyn Target>> = vec![Box::new(OutputTarget::new(target))];
        if verbose {
            targets.push(Box::new(MonitorTarget::new("outgoing")));
        }

        let input_name = self
            .input
            .port_name(&source)
            .map_err(Self::forwarding_error)?;

        let mut output_names: Vec<String> = Vec::new();
        for target in targets.iter() {
            if let Some(name) = target.describe(&output)? {
                output_names.push(name);
            }
        }

        println!("Activating route:");
        println!("  Input port: {}", input_name);
        if output_names.is_empty() {
            println!("  Output port: <none>");
        } else {
            println!("  Output port(s): {}", output_names.join(", "));
        }

        let Devices { input } = self;
        Route { source, targets }.activate(input, output)
    }

    pub fn monitor(self, source_name: String) -> Result<(), Errors> {
        let source = self
            .find_input_port(&source_name)
            .ok_or(Errors::InvalidInputPort(source_name))?;

        let input_name = self
            .input
            .port_name(&source)
            .map_err(Self::forwarding_error)?;
        println!("Monitoring input port: {}", input_name);

        let output = MidiOutput::new("MIDI Output").map_err(|_| Errors::InitFailure)?;
        let Devices { input } = self;
        Route {
            source,
            targets: vec![Box::new(MonitorTarget::new("incoming"))],
        }
        .activate(input, output)
    }

    fn find_input_port(&self, port_name: &str) -> Option<MidiInputPort> {
        self.input
            .ports()
            .into_iter()
            .find(|port| self.input.port_name(port) == Ok(port_name.to_string()))
    }

    fn find_output_port(output: &MidiOutput, port_name: &str) -> Option<MidiOutputPort> {
        output
            .ports()
            .into_iter()
            .find(|port| output.port_name(port) == Ok(port_name.to_string()))
    }

    fn forwarding_error<E: std::fmt::Display>(err: E) -> Errors {
        Errors::ForwardingError(err.to_string())
    }
}
