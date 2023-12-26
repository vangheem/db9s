use crate::app::Application;
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::{sync::Arc, sync::RwLock};

use crate::ui::state::LayoutState;
use crate::ui::types::WindowType;

pub struct InputBar {
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
    pub active: bool,
    input: String,
}

impl InputBar {
    pub fn new(app: Arc<Application>, state: Arc<RwLock<LayoutState>>) -> InputBar {
        InputBar {
            app,
            state,
            active: false,
            input: String::new(),
        }
    }

    fn select_window(&mut self) -> bool {
        match self.input.as_str() {
            ":connections" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(WindowType::ConnectionList);
                self.input.clear();
                true
            }
            ":conns" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(WindowType::ConnectionList);
                self.input.clear();
                true
            }
            ":tables" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(WindowType::TableList);
                self.input.clear();
                true
            }
            ":query" => {
                self.state.write().unwrap().change_window(WindowType::Query);
                self.input.clear();
                true
            }
            ":schemas" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(WindowType::SchemaList);
                self.input.clear();
                true
            }
            ":databases" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(WindowType::DatabaseList);
                self.input.clear();
                true
            }
            ":columns" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(WindowType::ColumnList);
                self.input.clear();
                true
            }
            _ => false,
        }
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char(c) => {
                    if self.active {
                        if c == ':' && self.input == ":".to_string() {
                            return;
                        }
                        self.input.push(c);
                    } else if c == ':' {
                        self.active = true;
                        self.input.push(c);
                        return;
                    }
                }
                KeyCode::Tab => {
                    if ":tables".starts_with(&self.input) {
                        self.input = ":tables".to_string();
                    }
                    if ":connections".starts_with(&self.input) {
                        self.input = ":connections".to_string();
                    }
                    if ":query".starts_with(&self.input) {
                        self.input = ":query".to_string();
                    }
                    if ":schemas".starts_with(&self.input) {
                        self.input = ":schemas".to_string();
                    }
                    if ":databases".starts_with(&self.input) {
                        self.input = ":databases".to_string();
                    }
                    if ":columns".starts_with(&self.input) {
                        self.input = ":columns".to_string();
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
                    if self.input.is_empty() {
                        self.active = false;
                    }
                }
                KeyCode::Enter => {
                    if self.select_window() {
                        self.active = false;
                    }
                }

                KeyCode::Esc => {
                    self.input.clear();
                    self.active = false;
                }
                _ => {}
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, rect: Rect, event: Option<Event>) {
        if let Some(event) = event {
            self.handle_event(event);
        }

        let para = Paragraph::new(Text::styled(
            self.input.clone(),
            Style::default().fg(Color::Green),
        ))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default()),
        );

        frame.render_widget(para, rect);
    }
}
