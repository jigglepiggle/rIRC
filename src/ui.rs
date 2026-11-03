use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;

pub fn render(frame: &mut Frame, app: &mut App) {
    // Split screen horizontally into 3 panels
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(18), // Channels
            Constraint::Min(20),    // Chat
            Constraint::Length(18), // Nicks
        ])
        .split(frame.area());

    let channels_area = horizontal[0];
    let chat_area = horizontal[1];
    let nicks_area = horizontal[2];

    render_channels(frame, channels_area);
    render_chat(frame, chat_area, app);
    render_nicks(frame, nicks_area);
}

fn render_channels(frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new("").block(
        Block::default()
            .title(" Channels ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );

    // Find a way to scan for channels and list them here
    // IRC Command: LIST

    frame.render_widget(widget, area);
}

fn render_nicks(frame: &mut Frame, area: Rect) {
    let widget = Paragraph::new("").block(
        Block::default()
            .title(" Nicks ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Blue)),
    );

    // Find a way to list all nicks in a channel and list them here
    // IRC Command: NAMES, WHOIS, WHO

    frame.render_widget(widget, area);
}

fn render_chat(frame: &mut Frame, area: Rect, app: &mut App) {
    // Split vertically into output & input
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(area);

    render_output(frame, vertical[0], app);
    render_input(frame, vertical[1], app);
}

fn render_output(frame: &mut Frame, area: Rect, app: &mut App) {
    let output = app.output().join("\n");

    // Calculate rendered line count, including approximate wrapping.
    let available_width = area.width.saturating_sub(2).max(1) as usize;

    let content_height: usize = output
        .lines()
        .map(|line| {
            let length = line.chars().count();
            length.max(1).div_ceil(available_width)
        })
        .sum();

    let visible_height = area.height.saturating_sub(2) as usize;

    if app.auto_scroll {
        app.scroll = content_height
            .saturating_sub(visible_height)
            .min(u16::MAX as usize) as u16;
    }

    let widget = Paragraph::new(Text::raw(output))
        .block(
            Block::default()
                .title(" Chat ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.scroll, 0));

    frame.render_widget(widget, area);
}

fn render_input(frame: &mut Frame, area: Rect, app: &App) {
    let widget = Paragraph::new(app.input()).block(
        Block::default()
            .title(" Input ")
            .borders(Borders::ALL)
            .border_style(
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    frame.render_widget(widget, area);

    let cursor_x = area.x + 1 + app.input().chars().count() as u16;
    let cursor_x = cursor_x.min(area.x + area.width.saturating_sub(2));

    frame.set_cursor_position((cursor_x, area.y + 1));
}
