use midir::{MidiInput, MidiInputPort, MidiOutput, MidiOutputPort};

use crate::Errors;
use crate::midi::route::Route;
use crate::midi::target::{MonitorTarget, OutputTarget, Target};

pub fn print() -> Result<(), Errors> {
    let input = MidiInput::new("midi-tool").map_err(|_| Errors::InitFailure)?;
    let output = MidiOutput::new("midi-tool").map_err(|_| Errors::InitFailure)?;

    if input.ports().is_empty() {
        println!("No source ports found.");
    } else {
        println!("Source ports: ");

        input.ports().iter().enumerate().for_each(|(i, port)| {
            let name = input
                .port_name(port)
                .unwrap_or_else(|_| "<unknown>".to_string());
            println!("{}: {}", i, name)
        });
    }

    if output.ports().is_empty() {
        println!("No target ports found.");
    } else {
        println!("Target ports: ");

        output.ports().iter().enumerate().for_each(|(i, port)| {
            let name = output
                .port_name(port)
                .unwrap_or_else(|_| "<unknown>".to_string());
            println!("{}: {}", i, name)
        });
    }

    Ok(())
}

pub fn route(source: String, target: String, verbose: bool) -> Result<(), Errors> {
    let source_port = find_source_port(&source)?;
    let target_port = find_target_port(&target)?;

    let route_targets: Vec<Box<dyn Target>> = if verbose {
        vec![
            Box::new(OutputTarget::new(target, target_port)),
            Box::new(MonitorTarget::new(source.to_string())),
        ]
    } else {
        vec![Box::new(OutputTarget::new(target, target_port))]
    };

    let target_names: String = route_targets
        .iter()
        .filter_map(|target| target.describe())
        .collect::<Vec<String>>()
        .join(", ");

    println!("Activating route:");
    println!("  Source: {}", source);
    println!("  Target(s): {}", target_names);

    Route {
        source: source_port,
        targets: route_targets,
    }
    .activate()
}

pub fn monitor(source_name: String) -> Result<(), Errors> {
    let source = find_source_port(&source_name)?;

    println!("Monitoring source port: {}", source_name);

    Route {
        source,
        targets: vec![Box::new(MonitorTarget::new(source_name.to_string()))],
    }
    .activate()
}

fn find_source_port(port_name: &str) -> Result<MidiInputPort, Errors> {
    let input = MidiInput::new("midi-tool").map_err(|_| Errors::InitFailure)?;

    input
        .ports()
        .into_iter()
        .find(|port| input.port_name(port) == Ok(port_name.to_string()))
        .ok_or(Errors::InvalidSourcePort(port_name.to_string()))
}

fn find_target_port(port_name: &str) -> Result<MidiOutputPort, Errors> {
    let output = MidiOutput::new("midi-tool").map_err(|_| Errors::InitFailure)?;

    output
        .ports()
        .into_iter()
        .find(|port| output.port_name(port) == Ok(port_name.to_string()))
        .ok_or(Errors::InvalidTargetPort(port_name.to_string()))
}
