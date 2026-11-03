use std::io;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

use ratatui::DefaultTerminal;

use crate::commands::{self, CommandAction};
use crate::ui;

pub struct App {
    input: String,
    output: Vec<String>,
    should_quit: bool,

    // IRC connection state
    tx: Sender<String>,
    current_channel: Option<String>,

    pub scroll: u16,
    pub auto_scroll: bool,
}

impl App {
    pub fn new(tx: Sender<String>) -> Self {
        Self {
            input: String::new(),
            output: Vec::new(),
            should_quit: false,
            tx,
            current_channel: None,
            scroll: u16::MIN,
            auto_scroll: true,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal, rx: Receiver<String>) -> io::Result<()> {
        while !self.should_quit {
            // Receive incoming IRC messages.
            while let Ok(message) = rx.try_recv() {
                self.push_message(message);
            }

            // Render the interface.
            terminal.draw(|frame| ui::render(frame, self))?;

            // Poll keyboard without blocking IRC message reception.
            if event::poll(Duration::from_millis(10))? {
                self.handle_event(event::read()?)?;
            }
        }

        Ok(())
    }

    pub fn push_message(&mut self, message: String) {
        self.output.push(message);

        if self.auto_scroll {
            self.scroll = u16::MAX;
        }
    }

    pub fn output(&self) -> &[String] {
        &self.output
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    fn handle_event(&mut self, event: Event) -> io::Result<()> {
        if let Event::Key(key) = event {
            if key.kind != KeyEventKind::Press {
                return Ok(());
            }

            match key.code {
                // Exit
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.should_quit = true;
                }

                // Clear input or exit
                KeyCode::Esc => {
                    self.input.clear();
                }

                // Type into input
                KeyCode::Char(c) => {
                    self.input.push(c);
                }

                // Delete character
                KeyCode::Backspace => {
                    self.input.pop();
                }

                // Submit input
                KeyCode::Enter => {
                    self.submit_input();
                }

                KeyCode::Up => {
                    self.auto_scroll = false;
                    self.scroll = self.scroll.saturating_sub(3);
                }

                KeyCode::Down => {
                    self.scroll = self.scroll.saturating_add(3);
                }

                KeyCode::PageUp => {
                    self.auto_scroll = false;
                    self.scroll = u16::MIN;
                }

                KeyCode::PageDown => {
                    self.auto_scroll = true;
                    self.scroll = u16::MAX;
                }

                _ => {}
            }
        }

        Ok(())
    }

    fn submit_input(&mut self) {
        let input = std::mem::take(&mut self.input);

        if input.trim().is_empty() {
            return;
        }

        match commands::handle_input(&input, self.current_channel.as_deref()) {
            Ok(CommandAction::SendIrc(message)) => match self.tx.send(message) {
                Ok(()) => {
                    self.push_message(format!("> {input}"));
                }
                Err(_) => {
                    self.push_message("ERROR: IRC writer disconnected".to_string());
                }
            },

            Ok(CommandAction::Join(channel)) => match self.tx.send(format!("JOIN {channel}")) {
                Ok(()) => {
                    self.current_channel = Some(channel.clone());
                    self.push_message(format!("Joining {channel}..."));
                }
                Err(_) => {
                    self.push_message("ERROR: IRC writer disconnected".to_string());
                }
            },

            Ok(CommandAction::Local(message)) => {
                self.push_message(message);
            }

            Ok(CommandAction::Quit) => {
                let _ = self.tx.send("QUIT :Leaving IRC".to_string());
                self.should_quit = true;
            }

            Err(error) => {
                self.push_message(format!("ERROR: {error}"));
            }
        }
    }
}
