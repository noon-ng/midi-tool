use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};

use crate::Errors;

pub(crate) trait Outgoing: Send {
    fn send(&mut self, message: &[u8]) -> Result<(), String>;
}

pub(crate) trait Target {
    fn describe(&self, output: &MidiOutput) -> Result<Option<String>, Errors>;
    fn connect(self: Box<Self>, output: MidiOutput) -> Result<Box<dyn Outgoing + Send>, Errors>;
}

pub(crate) struct OutputTarget {
    port: MidiOutputPort,
}

impl OutputTarget {
    pub(crate) fn new(port: MidiOutputPort) -> Self {
        Self { port }
    }
}

impl Target for OutputTarget {
    fn describe(&self, output: &MidiOutput) -> Result<Option<String>, Errors> {
        let name = output.port_name(&self.port).map_err(forwarding_error)?;
        Ok(Some(name))
    }

    fn connect(self: Box<Self>, output: MidiOutput) -> Result<Box<dyn Outgoing + Send>, Errors> {
        let connection = output
            .connect(&self.port, "midi-router")
            .map_err(forwarding_error)?;
        Ok(Box::new(MidiOutgoing { connection }))
    }
}

pub(crate) struct MonitorTarget {
    label: &'static str,
}

impl MonitorTarget {
    pub(crate) fn new(label: &'static str) -> Self {
        Self { label }
    }
}

impl Target for MonitorTarget {
    fn describe(&self, _output: &MidiOutput) -> Result<Option<String>, Errors> {
        Ok(None)
    }

    fn connect(self: Box<Self>, _output: MidiOutput) -> Result<Box<dyn Outgoing + Send>, Errors> {
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

fn forwarding_error<E: std::fmt::Display>(err: E) -> Errors {
    Errors::ForwardingError(err.to_string())
}
