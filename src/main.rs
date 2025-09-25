use arboard::Clipboard;
use chrono::Local;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use dirs::config_dir;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem, ListState},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::spawn;
use std::{thread, time::Duration};

#[derive(Serialize, Deserialize, Clone)]
struct Entry {
    timestamp: String,
    content: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    max_history: usize,
    poll_interval_ms: u64,
}

enum InputMode {
    Normal,
    Searching(String),
}

fn load_config() -> Config {
    let mut path = config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("clipman/config.json");

    if path.exists() {
        let data = fs::read_to_string(&path).unwrap();
        serde_json::from_str(&data).unwrap_or(Config {
            max_history: 200,
            poll_interval_ms: 300,
        })
    } else {
        Config {
            max_history: 200,
            poll_interval_ms: 300,
        }
    }
}

fn get_history_path() -> PathBuf {
    let mut path = config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("clipman");
    fs::create_dir_all(&path).unwrap();
    path.push("history.json");
    path
}

fn load_history() -> Vec<Entry> {
    let path = get_history_path();
    if path.exists() {
        let data = fs::read_to_string(path).unwrap_or_default();
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    }
}

fn save_history(history: &Vec<Entry>) {
    let path = get_history_path();
    let data = serde_json::to_string_pretty(history).unwrap();
    fs::write(path, data).unwrap();
}

fn to_list_item(e: &Entry) -> ListItem<'_> {
    let display = if e.content.trim().is_empty() {
        format!("(whitespace: {:?})", e.content)
    } else {
        e.content.clone()
    };
    ListItem::new(format!("[{}] {}", e.timestamp, display))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let history = Arc::new(Mutex::new(load_history()));
    let history_clone = Arc::clone(&history);
    let config = load_config();
    let mut input_mode = InputMode::Normal;

    spawn(move || {
        let mut clipboard = Clipboard::new().unwrap();
        let mut last_text: Option<String> = None;

        loop {
            match clipboard.get_text() {
                Ok(current_text) => {
                    let is_only_newline = current_text
                        .chars()
                        .all(|c| c == '\n' || c == '\r' || c == '\r');

                    if is_only_newline {
                        thread::sleep(Duration::from_millis(config.poll_interval_ms));
                        continue;
                    }

                    if last_text.as_ref() != Some(&current_text) {
                        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                        last_text = Some(current_text.clone());

                        std::io::stdout().flush().unwrap();

                        let mut hist = history_clone.lock().unwrap();
                        hist.push(Entry {
                            timestamp,
                            content: current_text.clone(),
                        });

                        let max_history = config.max_history;

                        if hist.len() > max_history {
                            hist.remove(0);
                        }

                        save_history(&hist);
                    }
                }
                Err(e) => eprintln!("Error accessing clipboard: {}", e),
            }
            thread::sleep(Duration::from_millis(config.poll_interval_ms));
        }
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut list_state = ListState::default();
    list_state.select(Some(0));

    loop {
        let hist = history.lock().unwrap().clone();

        terminal.draw(|f| {
            let size = f.area();
            let title = match &input_mode {
                InputMode::Normal => format!("Clipboard History ({} items)", hist.len()),
                InputMode::Searching(query) => format!("Search: {}", query),
            };

            let items: Vec<ListItem> = match &input_mode {
                InputMode::Normal => hist.iter().rev().map(|e| to_list_item(e)).collect(),

                InputMode::Searching(query) => hist
                    .iter()
                    .rev()
                    .filter(|e| e.content.contains(query))
                    .map(|e| to_list_item(e))
                    .collect(),
            };

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_symbol(">>");

            f.render_stateful_widget(list, size, &mut list_state);
        })?;

        if event::poll(Duration::from_millis(200))? {
            if let CEvent::Key(key) = event::read()? {
                let len = history.lock().unwrap().len();
                let mut selected = list_state.selected().unwrap_or(0);

                match input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => break,
                        KeyCode::Down => {
                            if selected + 1 < len {
                                selected += 1;
                            }
                            list_state.select(Some(selected));
                        }
                        KeyCode::Up => {
                            if selected > 0 {
                                selected -= 1;
                            }
                            list_state.select(Some(selected));
                        }
                        KeyCode::Enter => {
                            if let Some(idx) = list_state.selected() {
                                if let Some(entry) = history.lock().unwrap().get(len - 1 - idx) {
                                    let mut clipboard = Clipboard::new().unwrap();
                                    clipboard.set_text(entry.content.clone()).unwrap();
                                }
                            }
                        }
                        KeyCode::Char('/') => {
                            input_mode = InputMode::Searching(String::new());
                        }
                        _ => {}
                    },

                    InputMode::Searching(ref mut query) => match key.code {
                        KeyCode::Esc => {
                            input_mode = InputMode::Normal;
                        }
                        KeyCode::Enter => {
                            input_mode = InputMode::Normal;
                        }
                        KeyCode::Char(c) => {
                            query.push(c);
                        }
                        KeyCode::Backspace => {
                            query.pop();
                        }
                        _ => {}
                    },
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
