use midir::{Ignore, MidiInput, MidiInputPort, MidiOutput};
use std::io::stdin;

use crate::Errors;
use crate::midi::target::{Outgoing, Target};

pub(crate) struct Route {
    pub source: MidiInputPort,
    pub targets: Vec<Box<dyn Target>>,
}

impl Route {
    pub(crate) fn activate(self, mut input: MidiInput, output: MidiOutput) -> Result<(), Errors> {
        let mut outgoing_connections: Vec<Box<dyn Outgoing + Send>> = Vec::new();
        let mut output = Some(output);
        for target in self.targets {
            let midi_out = match output.take() {
                Some(existing) => existing,
                None => MidiOutput::new("MIDI Output").map_err(|_| Errors::InitFailure)?,
            };
            outgoing_connections.push(target.connect(midi_out)?);
        }

        input.ignore(Ignore::None);

        let _connection = input
            .connect(
                &self.source,
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
            .map_err(forwarding_error)?;

        stdin().read_line(&mut String::new()).ok();
        Ok(())
    }
}

fn forwarding_error<E: std::fmt::Display>(err: E) -> Errors {
    Errors::ForwardingError(err.to_string())
}
