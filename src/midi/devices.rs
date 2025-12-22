use midir::{Ignore, MidiInput, MidiInputPort, MidiOutput, MidiOutputConnection, MidiOutputPort};
use std::io::stdin;

use crate::Errors;

pub struct Devices {
    input: MidiInput,
    output: MidiOutput,
}

trait Outgoing: Send {
    fn send(&mut self, message: &[u8]) -> Result<(), String>;
}

trait Target {
    fn describe(&self, output: &MidiOutput) -> Result<Option<String>, Errors>;
    fn connect(
        self: Box<Self>,
        output: &mut MidiOutputPool,
    ) -> Result<Box<dyn Outgoing + Send>, Errors>;
}

struct OutputTarget {
    port: MidiOutputPort,
}

impl OutputTarget {
    fn new(port: MidiOutputPort) -> Self {
        Self { port }
    }
}

impl Target for OutputTarget {
    fn describe(&self, output: &MidiOutput) -> Result<Option<String>, Errors> {
        let name = output
            .port_name(&self.port)
            .map_err(Devices::forwarding_error)?;
        Ok(Some(name))
    }

    fn connect(
        self: Box<Self>,
        output: &mut MidiOutputPool,
    ) -> Result<Box<dyn Outgoing + Send>, Errors> {
        let midi_out = output.acquire()?;
        let connection = midi_out
            .connect(&self.port, "midi-router")
            .map_err(Devices::forwarding_error)?;
        Ok(Box::new(MidiOutgoing { connection }))
    }
}

struct MonitorTarget {
    label: &'static str,
}

impl MonitorTarget {
    fn new(label: &'static str) -> Self {
        Self { label }
    }
}

impl Target for MonitorTarget {
    fn describe(&self, _output: &MidiOutput) -> Result<Option<String>, Errors> {
        Ok(None)
    }

    fn connect(
        self: Box<Self>,
        _output: &mut MidiOutputPool,
    ) -> Result<Box<dyn Outgoing + Send>, Errors> {
        Ok(Box::new(MonitorOutgoing::new(self.label)))
    }
}

struct MidiOutgoing {
    connection: MidiOutputConnection,
}

impl Outgoing for MidiOutgoing {
    fn send(&mut self, message: &[u8]) -> Result<(), String> {
        self.connection
            .send(message)
            .map_err(|e| format!("{:?}", e))
    }
}

struct MonitorOutgoing {
    label: &'static str,
}

impl MonitorOutgoing {
    fn new(label: &'static str) -> Self {
        Self { label }
    }
}

impl Outgoing for MonitorOutgoing {
    fn send(&mut self, message: &[u8]) -> Result<(), String> {
        println!("{} message: {:?}", self.label, message);
        Ok(())
    }
}

struct MidiOutputPool {
    output: Option<MidiOutput>,
}

impl MidiOutputPool {
    fn new(output: MidiOutput) -> Self {
        Self {
            output: Some(output),
        }
    }

    fn acquire(&mut self) -> Result<MidiOutput, Errors> {
        match self.output.take() {
            Some(existing) => Ok(existing),
            None => MidiOutput::new("MIDI Output").map_err(|_| Errors::InitFailure),
        }
    }
}

struct Route {
    pub source: MidiInputPort,
    pub targets: Vec<Box<dyn Target>>,
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
                    let name = self
                        .input
                        .port_name(port)
                        .unwrap_or_else(|_| "<unknown>".to_string());
                    println!("{}: {}", i, name)
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
                        let name = self
                            .output
                            .port_name(port)
                            .unwrap_or_else(|_| "<unknown>".to_string());
                        println!("{}: {}", i, name)
                    });
            }
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

        let target = self
            .find_output_port(&target_name)
            .ok_or(Errors::InvalidOutputPort(target_name))?;

        let mut targets: Vec<Box<dyn Target>> = vec![Box::new(OutputTarget::new(target))];
        if verbose {
            targets.push(Box::new(MonitorTarget::new("outgoing")));
        }

        self.activate(Route { source, targets })
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

    fn activate(self, route: Route) -> Result<(), Errors> {
        let input_name = self
            .input
            .port_name(&route.source)
            .map_err(Self::forwarding_error)?;

        let mut output_names: Vec<String> = Vec::new();
        for target in route.targets.iter() {
            if let Some(name) = target.describe(&self.output)? {
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

        self.connect_route(route)
    }

    fn forwarding_error<E: std::fmt::Display>(err: E) -> Errors {
        Errors::ForwardingError(err.to_string())
    }

    fn connect_route(mut self, route: Route) -> Result<(), Errors> {
        let mut outgoing_connections: Vec<Box<dyn Outgoing + Send>> = Vec::new();
        let mut output = MidiOutputPool::new(self.output);
        for target in route.targets {
            outgoing_connections.push(target.connect(&mut output)?);
        }

        self.input.ignore(Ignore::None);

        let _connection = self
            .input
            .connect(
                &route.source,
                "midi-router",
                move |_stamp, message, _| {
                    for connection in outgoing_connections.iter_mut() {
                        if let Err(e) = connection.send(message) {
                            println!("Error sending message: {}", e)
                        }
                    }
                },
                (),
            )
            .map_err(Self::forwarding_error)?;

        stdin().read_line(&mut String::new()).ok();
        Ok(())
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

        self.connect_route(Route {
            source,
            targets: vec![Box::new(MonitorTarget::new("incoming"))],
        })
    }
}
