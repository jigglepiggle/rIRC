use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::mpsc::Sender;

use crate::messages::{Event, classify, parse_line};

pub fn run(stream: TcpStream, writer_tx: Sender<String>) -> io::Result<()> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();

        if reader.read_line(&mut line)? == 0 {
            println!("Server disconnected.");
            break;
        }

        // messages.rs
        match parse_line(&line).map(classify) {
            Some(Event::ServerReply { code, params }) => println!("reply {code}: {params:?}"),

            Some(Event::ChatMessage {
                sender,
                target,
                text,
            }) => println!("{target} <{sender}> {text}"),

            Some(Event::Quit { nickname, reason }) => {
                println!("user <{nickname}> quit: {reason}")
            }

            Some(Event::Join { nickname, channel }) => {
                println!("user <{nickname}> has joined channel {channel}")
            }

            Some(Event::Notice { target, text }) => {
                println!("{target} NOTICE: {text}")
            }

            Some(Event::ServerError(message)) => eprintln!("ERROR: {message}"),

            Some(_) => {}
            None => println!("not IRC: {line}"),
        }

        //print!("{line}");
        io::stdout().flush()?;

        // Reply to server keepalive messages
        if let Some(payload) = line.strip_prefix("PING ") {
            let payload = payload.trim_end_matches(&['\r', '\n'][..]);

            writer_tx
                .send(format!("PONG {payload}"))
                .map_err(|_| io::Error::other("Writer stopped"))?;
        }
    }

    Ok(())
}
