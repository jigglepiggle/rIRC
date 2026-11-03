use std::io::{self, Write};
use std::net::TcpStream;
use std::sync::mpsc::Receiver;

pub fn run(stream: TcpStream, rx: Receiver<String>) -> io::Result<()> {
    let mut writer = stream;

    for message in rx {
        let wire_message = format!("{message}\r\n");
        writer.write_all(wire_message.as_bytes())?;
        writer.flush()?;
    }

    Ok(())
}
