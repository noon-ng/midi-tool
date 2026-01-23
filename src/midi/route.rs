use midir::{Ignore, MidiInput, MidiInputPort, MidiOutput};
use std::io::stdin;

use crate::Errors;
use crate::midi::target::{Outgoing, Target};

pub(crate) struct Route {
    pub source: MidiInputPort,
    pub targets: Vec<Box<dyn Target>>,
}

impl Route {
    pub(crate) fn activate(self) -> Result<(), Errors> {
        let mut input = MidiInput::new("midi-tool").map_err(|_| Errors::InitFailure)?;

        input.ignore(Ignore::None);

        let mut outgoing_connections: Vec<Box<dyn Outgoing + Send>> = self
            .targets
            .into_iter()
            .map(|target| {
                target.connect(MidiOutput::new("midi-tool").map_err(|_| Errors::InitFailure)?)
            })
            .collect::<Result<_, _>>()?;

        let _connection = input.connect(
            &self.source,
            "midi-tool",
            move |_stamp, message, _| {
                for connection in outgoing_connections.iter_mut() {
                    if let Err(e) = connection.send(message) {
                        println!("Error sending message: {}", e)
                    }
                }
            },
            (),
        );

        stdin().read_line(&mut String::new()).ok();

        Ok(())
    }
}
