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
    let mut notes = read_notes()?;
    let mut list_state = ListState::default();
    if !notes.is_empty() {
        list_state.select(Some(0));
    }

    // Search state
    let mut search_mode = false;
    let mut search_term = String::new();
    let mut filtered_notes = notes.clone();

    // Main event loop
    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
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

            // Search bar
            let search_style = if search_mode {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let search_bar = Paragraph::new(Text::from(format!(
                "Search: {}",
                if search_mode { &search_term } else { "Press '/' to search" }
            )))
            .block(Block::default().borders(Borders::ALL))
            .style(search_style);
            f.render_widget(search_bar, chunks[1]);

            // Notes list
            let items: Vec<ListItem> = filtered_notes
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
            f.render_stateful_widget(list, chunks[2], &mut list_state);

            // Help text
            let help = Paragraph::new(Text::from(vec![
                Line::from(vec![
                    Span::styled("↑↓", Style::default().fg(Color::Yellow)),
                    Span::raw(" to navigate, "),
                    Span::styled("d", Style::default().fg(Color::Yellow)),
                    Span::raw(" to delete, "),
                    Span::styled("/", Style::default().fg(Color::Yellow)),
                    Span::raw(" to search, "),
                    Span::styled("q", Style::default().fg(Color::Yellow)),
                    Span::raw(" to quit"),
                ]),
            ]))
            .block(Block::default().borders(Borders::ALL));
            f.render_widget(help, chunks[3]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('/') => {
                    search_mode = true;
                    search_term.clear();
                    filtered_notes = notes.clone();
                }
                KeyCode::Esc => {
                    search_mode = false;
                    search_term.clear();
                    filtered_notes = notes.clone();
                    if !filtered_notes.is_empty() {
                        list_state.select(Some(0));
                    }
                }
                KeyCode::Backspace => {
                    if search_mode {
                        search_term.pop();
                        update_filtered_notes(&notes, &search_term, &mut filtered_notes);
                        if !filtered_notes.is_empty() {
                            list_state.select(Some(0));
                        }
                    }
                }
                KeyCode::Char(c) => {
                    if search_mode {
                        search_term.push(c);
                        update_filtered_notes(&notes, &search_term, &mut filtered_notes);
                        if !filtered_notes.is_empty() {
                            list_state.select(Some(0));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = list_state.selected() {
                        if selected > 0 {
                            list_state.select(Some(selected - 1));
                        }
                    }
                }
                KeyCode::Down => {
                    if let Some(selected) = list_state.selected() {
                        if selected < filtered_notes.len() - 1 {
                            list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Char('d') => {
                    if let Some(selected) = list_state.selected() {
                        if let Some(original_index) = notes.iter().position(|n| n == &filtered_notes[selected]) {
                            delete_note(original_index)?;
                            notes = read_notes()?;
                            update_filtered_notes(&notes, &search_term, &mut filtered_notes);
                            if filtered_notes.is_empty() {
                                list_state.select(None);
                            } else if selected >= filtered_notes.len() {
                                list_state.select(Some(filtered_notes.len() - 1));
                            }
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

fn update_filtered_notes(notes: &[String], search_term: &str, filtered_notes: &mut Vec<String>) {
    if search_term.is_empty() {
        *filtered_notes = notes.to_vec();
    } else {
        *filtered_notes = notes
            .iter()
            .filter(|note| {
                let parts: Vec<&str> = note.splitn(3, ']').collect();
                let category = parts.get(1).map(|s| s.trim_start_matches('[')).unwrap_or("");
                let content = parts.get(2).map(|s| s.trim()).unwrap_or("");
                category.to_lowercase().contains(&search_term.to_lowercase())
                    || content.to_lowercase().contains(&search_term.to_lowercase())
            })
            .cloned()
            .collect();
    }
}

fn read_notes() -> io::Result<Vec<String>> {
    if !Path::new(NOTES_FILE).exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(NOTES_FILE)?;
    Ok(content.lines().map(String::from).collect())
}

fn delete_note(index: usize) -> io::Result<()> {
    let mut notes = read_notes()?;
    if index < notes.len() {
        notes.remove(index);
        fs::write(NOTES_FILE, notes.join("\n"))?;
    }
    Ok(())
} 