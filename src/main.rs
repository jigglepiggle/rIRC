use std::io::{self, BufRead};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

mod commands;
use commands::{CommandAction, handle_input};

mod messages;
mod reader;
mod writer;

fn main() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:6667")?;
    println!("Connected to IRC server!");

    // Create queue for outgoing IRC commands
    let (tx, rx) = mpsc::channel::<String>();

    // Clone stream for reader tx/rx
    let reader_stream = stream.try_clone()?;

    // Thread 1: Writer
    let writer_handle = thread::spawn(move || writer::run(stream, rx));

    // Queue registration commands before starting normal input
    tx.send("NICK jigpig".to_string())
        .map_err(|_| io::Error::other("Writer stopped"))?;
    tx.send("USER jigpig 0 * :Rust IRC Client".to_string())
        .map_err(|_| io::Error::other("Writer stopped"))?;

    // Give reader it's own sender / reader
    let reader_tx = tx.clone();

    // Thread 2: Reader
    let reader_handle = thread::spawn(move || reader::run(reader_stream, reader_tx));

    // Thread 3: Input
    let input_handle = thread::spawn(move || -> io::Result<()> {
        let stdin = io::stdin();
        let mut current_channel: Option<String> = None;

        for line in stdin.lock().lines() {
            let line = line?;

            match handle_input(&line, current_channel.as_deref()) {
                Ok(CommandAction::SendIrc(message)) => {
                    tx.send(message)
                        .map_err(|_| io::Error::other("Writer stopped"))?;
                }

                Ok(CommandAction::Join(channel)) => {
                    tx.send(format!("JOIN {channel}"))
                        .map_err(|_| io::Error::other("Writer stopped"))?;

                    current_channel = Some(channel);
                }

                Ok(CommandAction::Local(message)) => {
                    if !message.is_empty() {
                        println!("{message}");
                    }
                }

                Ok(CommandAction::Quit) => {
                    tx.send("QUIT :Leaving IRC".to_string())
                        .map_err(|_| io::Error::other("Writer stopped"))?;
                    break;
                }

                Err(error) => {
                    println!("Error: {error}");
                }
            }
        }

        Ok(())
    });

    // Keep main thread alive whilst workers run
    reader_handle
        .join()
        .map_err(|_| io::Error::other("Reader thread panicked"))??;

    // Server has disconnected, Shut down workers
    drop(input_handle);
    writer_handle
        .join()
        .map_err(|_| io::Error::other("Writer thread panicked"))??;

    Ok(())
}
