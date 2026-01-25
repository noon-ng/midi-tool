use midir::{MidiInput, MidiInputPort, MidiOutput, MidiOutputPort};

use crate::Errors;
use crate::midi::route::Route;
use crate::midi::target::{MonitorTarget, OutputTarget, Target};

pub fn print() -> Result<(), Errors> {
    let input = MidiInput::new("midi-tool").map_err(|_| Errors::InitFailure)?;
    let output = MidiOutput::new("midi-tool").map_err(|_| Errors::InitFailure)?;

    if input.ports().len() == 0 {
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

    if output.ports().len() == 0 {
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

pub fn route(source_name: String, target_name: String, verbose: bool) -> Result<(), Errors> {
    let source = find_input_port(&source_name)?;
    let target = find_output_port(&target_name)?;

    let targets: Vec<Box<dyn Target>> = if verbose {
        vec![
            Box::new(OutputTarget::new(target_name, target)),
            Box::new(MonitorTarget::new(source_name.to_string())),
        ]
    } else {
        vec![Box::new(OutputTarget::new(target_name, target))]
    };

    let target_names: String = targets
        .iter()
        .filter_map(|target| target.describe())
        .collect::<Vec<String>>()
        .join(", ");

    println!("Activating route:");
    println!("  Source: {}", source_name);
    println!("  Target(s): {}", target_names);

    Route { source, targets }.activate()
}

pub fn monitor(source_name: String) -> Result<(), Errors> {
    let source = find_input_port(&source_name)?;

    println!("Monitoring source port: {}", source_name);

    Route {
        source,
        targets: vec![Box::new(MonitorTarget::new(source_name.to_string()))],
    }
    .activate()
}

fn find_input_port(port_name: &str) -> Result<MidiInputPort, Errors> {
    let input = MidiInput::new("midi-tool").map_err(|_| Errors::InitFailure)?;

    input
        .ports()
        .into_iter()
        .find(|port| input.port_name(port) == Ok(port_name.to_string()))
        .ok_or(Errors::InvalidSourcePort(port_name.to_string()))
}

fn find_output_port(port_name: &str) -> Result<MidiOutputPort, Errors> {
    let output = MidiOutput::new("midi-tool").map_err(|_| Errors::InitFailure)?;

    output
        .ports()
        .into_iter()
        .find(|port| output.port_name(port) == Ok(port_name.to_string()))
        .ok_or(Errors::InvalidTargetPort(port_name.to_string()))
}
