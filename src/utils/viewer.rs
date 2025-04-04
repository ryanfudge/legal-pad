use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style}, // may need to add Stylize
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState},
    Terminal,
};
use std::{
    fs,
    io::{self, stdout},
    path::Path,
};

const NOTES_FILE: &str = "notes.txt";

pub fn view_notes() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    // Read notes
    let notes = read_notes()?;
    let mut list_state = ListState::default();
    if !notes.is_empty() {
        list_state.select(Some(0));
    }

    // Main event loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(1),
                    Constraint::Length(3),
                ])
                .split(f.size());

            // Header
            let header = Block::default()
                .title("Legal Pad")
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Yellow));
            f.render_widget(header, chunks[0]);

            // Notes list
            let items: Vec<ListItem> = notes
                .iter()
                .map(|note| {
                    // Split the note into parts
                    let parts: Vec<&str> = note.splitn(3, ']').collect();
                    let timestamp = parts.get(0).map(|s| s.trim_start_matches('[')).unwrap_or("");
                    let category = parts.get(1).map(|s| s.trim_start_matches('[')).unwrap_or("");
                    let content = parts.get(2).map(|s| s.trim()).unwrap_or("");

                    let timestamp = Span::styled(
                        format!("[{}]", timestamp),
                        Style::default().fg(Color::Cyan),
                    );
                    let category = Span::styled(
                        format!("[{}]", category),
                        Style::default().fg(Color::Green),
                    );
                    let content = Span::raw(content);
                    ListItem::new(Line::from(vec![
                        timestamp,
                        Span::raw(" "),
                        category,
                        Span::raw(" "),
                        content,
                    ]))
                })
                .collect();

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL))
                .highlight_style(Style::default().bg(Color::DarkGray))
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, chunks[1], &mut list_state);

            // Help text
            let help = Paragraph::new(Text::from(vec![
                Line::from(vec![
                    Span::styled("q", Style::default().fg(Color::Yellow)),
                    Span::raw(" to quit, "),
                    Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                    Span::raw(" to navigate"),
                ]),
            ]))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(help, chunks[2]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Up => {
                    if let Some(selected) = list_state.selected() {
                        if selected > 0 {
                            list_state.select(Some(selected - 1));
                        }
                    }
                }
                KeyCode::Down => {
                    if let Some(selected) = list_state.selected() {
                        if selected < notes.len() - 1 {
                            list_state.select(Some(selected + 1));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

fn read_notes() -> io::Result<Vec<String>> {
    if !Path::new(NOTES_FILE).exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(NOTES_FILE)?;
    Ok(content.lines().map(String::from).collect())
} 