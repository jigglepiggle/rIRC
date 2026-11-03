use std::io;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;

mod commands;
mod messages;
mod reader;
mod writer;

mod app;
mod ui;

use app::App;

fn main() -> io::Result<()> {
    // Connect to IRC server.
    let stream = TcpStream::connect("127.0.0.1:6667")?;

    let shutdown_stream = stream.try_clone()?;
    let reader_stream = stream.try_clone()?;

    // Outgoing IRC command channel.
    let (tx, rx) = mpsc::channel::<String>();

    // Incoming messages for the TUI.
    let (ui_tx, ui_rx) = mpsc::channel::<String>();

    // Start writer thread.
    let writer_handle = thread::spawn(move || writer::run(stream, rx));

    // Register with IRC server.
    tx.send("NICK jigpig".to_string())
        .map_err(|_| io::Error::other("Writer stopped"))?;

    tx.send("USER jigpig 0 * :Rust IRC Client".to_string())
        .map_err(|_| io::Error::other("Writer stopped"))?;

    // Reader needs a sender for PONG responses.
    let reader_writer_tx = tx.clone();

    // Start reader thread.
    let reader_handle = thread::spawn(move || reader::run(reader_stream, reader_writer_tx, ui_tx));

    // Start TUI.
    let mut terminal = ratatui::init();

    let mut app = App::new(tx.clone());

    let result = app.run(&mut terminal, ui_rx);

    // Restore terminal before shutting down networking.
    ratatui::restore();

    // Unblock the reader without immediately closing the writer.
    let _ = shutdown_stream.shutdown(std::net::Shutdown::Read);

    drop(app);
    drop(tx);

    // Reader exits and releases its writer sender.
    let reader_result = reader_handle
        .join()
        .map_err(|_| io::Error::other("Reader thread panicked"))?;

    // Writer drains its remaining commands and exits.
    let writer_result = writer_handle
        .join()
        .map_err(|_| io::Error::other("Writer thread panicked"))?;

    let _ = shutdown_stream.shutdown(std::net::Shutdown::Write);

    result?;
    reader_result?;
    let _ = writer_result;

    Ok(())
}
