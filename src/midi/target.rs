use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use std::io::Write;
use std::sync::mpsc;
use std::thread;

use crate::Errors;
pub(crate) trait Outgoing: Send {
    fn send(&mut self, message: &[u8]) -> Result<(), String>;
}

pub(crate) trait Target {
    fn describe(&self) -> Option<String>;
    fn connect(self: Box<Self>, output: MidiOutput) -> Result<Box<dyn Outgoing + Send>, Errors>;
}

pub(crate) struct OutputTarget {
    name: String,
    port: MidiOutputPort,
}

impl OutputTarget {
    pub(crate) fn new(name: String, port: MidiOutputPort) -> Self {
        Self { name, port }
    }
}

impl Target for OutputTarget {
    fn describe(&self) -> Option<String> {
        Some(self.name.to_string())
    }

    fn connect(self: Box<Self>, output: MidiOutput) -> Result<Box<dyn Outgoing + Send>, Errors> {
        let connection = output
            .connect(&self.port, "midi-router")
            .map_err(|err| Errors::ForwardingError(err.to_string()))?;
        Ok(Box::new(MidiOutgoing { connection }))
    }
}

pub(crate) struct MonitorTarget {
    label: String,
}

impl MonitorTarget {
    pub(crate) fn new(label: String) -> Self {
        Self { label }
    }
}

impl Target for MonitorTarget {
    fn describe(&self) -> Option<String> {
        None
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
    tx: mpsc::Sender<Vec<u8>>,
}

impl MonitorOutgoing {
    fn new(label: String) -> Self {
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        let label = label.to_string();
        thread::spawn(move || {
            let mut last_len = 0usize;
            loop {
                match rx.recv() {
                    Ok(message) => {
                        last_len = print_monitor_line(&label, &message, last_len);
                    }
                    Err(_) => break,
                }
            }
        });

        Self { tx }
    }
}

impl Outgoing for MonitorOutgoing {
    fn send(&mut self, message: &[u8]) -> Result<(), String> {
        self.tx.send(message.to_vec()).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn decode_message(message: &[u8]) -> String {
    if message.is_empty() {
        return "empty message".to_string();
    }

    let status = message[0];
    if status == 0xF0 {
        let preview_len = usize::min(8, message.len());
        let preview = &message[..preview_len];
        return format!("SysEx len={} {:?}", message.len(), preview);
    }

    let (kind, channel) = match status & 0xF0 {
        0x80 => ("NoteOff", Some((status & 0x0F) + 1)),
        0x90 => ("NoteOn", Some((status & 0x0F) + 1)),
        0xA0 => ("PolyPressure", Some((status & 0x0F) + 1)),
        0xB0 => ("CC", Some((status & 0x0F) + 1)),
        0xC0 => ("Program", Some((status & 0x0F) + 1)),
        0xD0 => ("ChannelPressure", Some((status & 0x0F) + 1)),
        0xE0 => ("PitchBend", Some((status & 0x0F) + 1)),
        0xF0 => ("System", None),
        _ => ("Unknown", None),
    };

    let chan = channel.map(|c| format!(" ch{}", c)).unwrap_or_default();
    let detail = match status & 0xF0 {
        0x80 | 0x90 => {
            let note = message.get(1).copied().unwrap_or(0);
            let vel = message.get(2).copied().unwrap_or(0);
            format!(" note={} vel={}", note, vel)
        }
        0xA0 => {
            let note = message.get(1).copied().unwrap_or(0);
            let pressure = message.get(2).copied().unwrap_or(0);
            format!(" note={} pressure={}", note, pressure)
        }
        0xB0 => {
            let cc = message.get(1).copied().unwrap_or(0);
            let value = message.get(2).copied().unwrap_or(0);
            format!(" {}={}", cc, value)
        }
        0xC0 => {
            let program = message.get(1).copied().unwrap_or(0);
            format!(" program={}", program)
        }
        0xD0 => {
            let pressure = message.get(1).copied().unwrap_or(0);
            format!(" pressure={}", pressure)
        }
        0xE0 => {
            let lsb = message.get(1).copied().unwrap_or(0) as i16;
            let msb = message.get(2).copied().unwrap_or(0) as i16;
            let value = ((msb << 7) | lsb) - 8192;
            format!(" value={}", value)
        }
        _ => String::new(),
    };

    format!("{}{}{} {:?}", kind, chan, detail, message)
}

fn print_monitor_line(label: &str, message: &[u8], last_len: usize) -> usize {
    let decoded = decode_message(message);
    let mut line = format!("{}: {}", label, decoded);
    let max_len = 120usize;
    if line.len() > max_len {
        line.truncate(max_len.saturating_sub(3));
        line.push_str("...");
    }
    if line.len() < last_len {
        let padding = last_len - line.len();
        line.push_str(&" ".repeat(padding));
    }
    print!("\r{}", line);
    std::io::stdout().flush().ok();
    line.len()
}
