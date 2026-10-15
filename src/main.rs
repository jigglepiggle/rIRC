use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:6667")?;
    println!("Connected to IRC server!");

    stream.write_all(b"NICK rustclient\r\nUSER rustclient 0 * :Rust IRC Client\r\n",)?;

    let reader_stream = stream.try_clone()?;
    let mut reader = BufReader::new(reader_stream);
    let mut line = String::new();

    let mut join_sent = false;

    loop {
        line.clear();

        if reader.read_line(&mut line)? == 0 {
            println!("Server disconnected.");
            break;
        }

        println!("{line}");
        io::stdout().flush()?;

        if let Some(payload) = line.strip_prefix("PING ") {
            write!(stream, "PONG {payload}")?;
            stream.flush()?;
        }

        if line.contains(" 001 ") && !join_sent {
            stream.write_all(b"JOIN #test\r\n")?;
            stream.flush()?;

            join_sent = true;
            println!("Requested to join #test");
        }
    }

    Ok(())
}
