#[allow(dead_code)]
use midir::{MidiInputPort, MidiOutputPort};


pub struct Route {
    pub source: MidiInputPort,
    pub target: MidiOutputPort,
}

impl Route {
    pub fn new(source: MidiInputPort, target: MidiOutputPort) -> Self {
        Self { source, target }
    }
}
