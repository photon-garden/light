use midir::{Ignore, MidiInput, MidiInputConnection};
use mpsc::Receiver;
use std::error::Error;
use std::io::{stdin, stdout, Write};
use std::sync::mpsc;

pub enum MessageType {
    NoteOn,
    NoteOff,
    Other,
}

pub struct Message {
    pub kind: MessageType,
    pub pitch: u8,
    pub timestamp: f32,
}

fn to_seconds(microseconds: u64) -> f32 {
    (microseconds / 1_000_000) as f32
}

impl Message {
    pub fn from_microseconds(
        first_timestamp: u64,
        timestamp: u64,
        message_bytes: &[u8],
    ) -> Message {
        let status = message_bytes[0];
        let pitch = message_bytes[1];
        let note_type = Message::get_type(status);

        let first_timestamp_in_seconds = to_seconds(first_timestamp);
        let timestamp_in_seconds = to_seconds(timestamp);
        let relative_timestamp = timestamp_in_seconds - first_timestamp_in_seconds;

        Message {
            timestamp: relative_timestamp,
            kind: note_type,
            pitch,
        }
    }

    fn get_type(status: u8) -> MessageType {
        match status {
            0x90..=0x9F => MessageType::NoteOn,
            0x80..=0x8F => MessageType::NoteOff,
            _ => MessageType::Other,
        }
    }
}

pub fn receive() -> (Receiver<Message>, MidiInputConnection<()>) {
    let (sender, receiver) = mpsc::channel();

    let connection = connect(sender).unwrap();

    (receiver, connection)
}

fn connect(sender: mpsc::Sender<Message>) -> Result<MidiInputConnection<()>, Box<dyn Error>> {
    let mut input = String::new();

    let mut midi_in = MidiInput::new("midir reading input")?;
    midi_in.ignore(Ignore::None);

    // Get an input port (read from console if multiple are available)
    let in_ports = midi_in.ports();
    let in_port = match in_ports.len() {
        0 => return Err("no input port found".into()),
        1 => {
            println!(
                "Choosing the only available input port: {}",
                midi_in.port_name(&in_ports[0]).unwrap()
            );
            &in_ports[0]
        }
        _ => {
            println!("\nAvailable input ports:");
            for (i, p) in in_ports.iter().enumerate() {
                println!("{}: {}", i, midi_in.port_name(p).unwrap());
            }
            print!("Please select input port: ");
            stdout().flush()?;
            let mut input = String::new();
            stdin().read_line(&mut input)?;
            in_ports
                .get(input.trim().parse::<usize>()?)
                .ok_or("invalid input port selected")?
        }
    };

    println!("\nOpening connection");
    let in_port_name = midi_in.port_name(in_port)?;

    let mut maybe_first_timestamp: Option<u64> = None;

    // conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    let conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        move |timestamp, message_bytes, _| {
            let first_timestamp: u64;
            if let Some(saved_first_timestamp) = maybe_first_timestamp {
                first_timestamp = saved_first_timestamp;
            } else {
                maybe_first_timestamp = Some(timestamp);
                first_timestamp = timestamp;
            }

            let message = Message::from_microseconds(first_timestamp, timestamp, message_bytes);
            sender.send(message).unwrap();
        },
        (),
    )?;

    println!(
        "Connection open, reading input from '{}' (press enter to exit) ...",
        in_port_name
    );

    input.clear();
    // stdin().read_line(&mut input)?; // wait for next enter key press

    // println!("Closing connection");
    Ok(conn_in)
}