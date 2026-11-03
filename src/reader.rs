use std::io::{self, BufRead, BufReader};
use std::net::TcpStream;
use std::sync::mpsc::Sender;

use crate::messages::{Event, classify, parse_line};

pub fn run(stream: TcpStream, writer_tx: Sender<String>, ui_tx: Sender<String>) -> io::Result<()> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    loop {
        line.clear();

        if reader.read_line(&mut line)? == 0 {
            let _ = ui_tx.send(format!("Server disconnected."));
            break;
        }

        // messages.rs
        match parse_line(&line).map(classify) {
            Some(Event::ServerReply { code, params }) => {
                let _ = ui_tx.send(format!("reply {code}: {params:?}"));
            }

            Some(Event::ChatMessage {
                sender,
                target,
                text,
            }) => {
                let _ = ui_tx.send(format!("{target} <{sender}> {text}"));
            }

            Some(Event::Quit { nickname, reason }) => {
                let _ = ui_tx.send(format!("user <{nickname}> quit: {reason}"));
            }

            Some(Event::Join { nickname, channel }) => {
                let _ = ui_tx.send(format!("user <{nickname}> has joined channel {channel}"));
            }

            Some(Event::Notice { target, text }) => {
                let _ = ui_tx.send(format!("{target} NOTICE: {text}"));
            }

            Some(Event::ServerError(message)) => {
                let _ = ui_tx.send(format!("ERROR: {message}"));
            }

            Some(Event::Unhandled(_)) => {
                let _ = ui_tx.send("Unhandled IRC message".to_string());
            }

            None => {}
        }

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
