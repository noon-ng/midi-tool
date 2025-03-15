# 🎹 `midi-tool`

CLI tool for routing MIDI messages between devices, based on `midir`. I wrote
this to help me control some of my music gear while learning Rust.

## Features

- MIDI port enumeration 
- MIDI routing 

Assuming you get the names of the ports right (do a `midi-tool list`
first), `midi-tool route` will start forwarding all MIDI events from one
port to the other. Play some notes on the source MIDI device and
hopefully the target device makes noises!

## Usage

Compile with `cargo build --release` and run the binary from the target.

### **List MIDI Ports**
```sh
midi-tool list
```
Shows all available MIDI input/output ports.

### **Route MIDI Messages**
```sh
midi-tool route --source-name "MIDI Controller" --target-name "Synth"
```
Forwards MIDI data between two devices.

**Tip:** If your port names have spaces, wrap them in quotes.

## Plan

I plan on following up with additional MIDI features that I think of as
I use this when producing and jamming, maybe come up with a little swiss
knife type thing. Other than that, a few enhancements that I am
considering at the moment - keeping in mind this is a learning project:

- An interactive mode, with maybe a ncurses-based TUI (do people still
  use that these days?) where you can just navigate with the keyboard to
  create routings.

- Multithreading so the same process can forward multiple active routes
  in parallel.

- OS-native frontends for MacOS and Linux (Ghostty-style)

## Acknowledgements

The CLI uses `clap`, which from what I could gather is a fairly standard
crate. Unsurprisingly so, since at least at the level of complexity we
have here the `#[derive(Parser)]` directive is a joy to use.

The MIDI stuff uses `midir`. In fact, the `route` subcommand is
basically a reimplementation of the `test_forward.rs` example bundled
with that crate. I worked up to parity using it as a reference while
navigating the foreignness of the language.

And on that note, `rustlings` was an invaluable resource to get up to speed.

## License

This project is licensed under the **MIT License**. See the [LICENSE](LICENSE) file for full details.

Copyright 2025, Nuno Correia (@noon-ng on GitHub)
