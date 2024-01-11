use crate::app::Application;
use crate::connectiontypes::utils::validate_dsn;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::Line;
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use ratatui_textarea::{CursorMove, TextArea};
use std::{sync::Arc, sync::RwLock};

use crate::ui::state::LayoutState;
use crate::ui::types;

trait InputReceiver {
    fn receive_input(&mut self, event: Event) -> bool;
    fn active(&self, event: Option<Event>) -> bool;
    fn clear(&mut self);
    fn layout_size(&self) -> u16;
    fn render(&mut self, frame: &mut Frame, rect: Rect);
}

struct CommandInputReceiver {
    input: String,
    active: bool,
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
}

impl CommandInputReceiver {
    pub fn new(app: Arc<Application>, state: Arc<RwLock<LayoutState>>) -> CommandInputReceiver {
        CommandInputReceiver {
            input: String::new(),
            active: false,
            app,
            state,
        }
    }

    fn select_window(&mut self) -> bool {
        match self.input.as_str() {
            ":connections" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(types::WindowTypeID::CONNECTIONS);
                self.input.clear();
                true
            }
            ":conns" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(types::WindowTypeID::CONNECTIONS);
                self.input.clear();
                true
            }
            ":tables" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(types::WindowTypeID::TABLES);
                self.input.clear();
                true
            }
            ":query" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(types::WindowTypeID::QUERY);
                self.input.clear();
                true
            }
            ":schemas" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(types::WindowTypeID::SCHEMAS);
                self.input.clear();
                true
            }
            ":databases" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(types::WindowTypeID::DATABASES);
                self.input.clear();
                true
            }
            ":columns" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(types::WindowTypeID::COLUMNS);
                self.input.clear();
                true
            }
            ":history" => {
                self.state
                    .write()
                    .unwrap()
                    .change_window(types::WindowTypeID::HISTORY);
                self.input.clear();
                true
            }
            _ => false,
        }
    }
}

impl InputReceiver for CommandInputReceiver {
    fn receive_input(&mut self, event: Event) -> bool {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char(c) => {
                    if self.active {
                        if c != ':' || self.input != ":".to_string() {
                            self.input.push(c);
                        }
                        return true;
                    } else if c == ':' {
                        self.active = true;
                        self.input.push(c);
                        return true;
                    }
                }
                KeyCode::Tab => {
                    if !self.active {
                        return false;
                    }
                    if ":tables".starts_with(&self.input) {
                        self.input = ":tables".to_string();
                    } else if ":connections".starts_with(&self.input) {
                        self.input = ":connections".to_string();
                    } else if ":query".starts_with(&self.input) {
                        self.input = ":query".to_string();
                    } else if ":schemas".starts_with(&self.input) {
                        self.input = ":schemas".to_string();
                    } else if ":databases".starts_with(&self.input) {
                        self.input = ":databases".to_string();
                    } else if ":columns".starts_with(&self.input) {
                        self.input = ":columns".to_string();
                    } else if ":history".starts_with(&self.input) {
                        self.input = ":history".to_string();
                    }
                }
                KeyCode::Backspace => {
                    if self.active {
                        self.input.pop();
                        if self.input.is_empty() {
                            self.clear();
                        }
                        return true;
                    }
                }
                KeyCode::Enter => {
                    if self.active {
                        if self.select_window() {
                            self.clear();
                        }
                        return true;
                    }
                }

                KeyCode::Esc => {
                    self.clear();
                }
                _ => {}
            }
        }
        false
    }
    fn active(&self, event: Option<Event>) -> bool {
        self.active
    }
    fn clear(&mut self) {
        self.input.clear();
        self.active = false;
    }

    fn render(&mut self, frame: &mut Frame, rect: Rect) {
        let lines = vec![Line::from(Span::styled(
            self.input.clone(),
            Style::default().fg(Color::Green),
        ))];
        let para = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title("Command"),
        );
        frame.render_widget(para, rect);
    }

    fn layout_size(&self) -> u16 {
        3
    }
}

struct ConnectionInputReceiver {
    input: String,
    name: String,
    name_validated: bool,
    dsn_validated: bool,
    dsn_validation_error: String,
    active: bool,
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
}

impl ConnectionInputReceiver {
    pub fn new(app: Arc<Application>, state: Arc<RwLock<LayoutState>>) -> ConnectionInputReceiver {
        ConnectionInputReceiver {
            name: String::new(),
            input: String::new(),
            active: false,
            name_validated: false,
            dsn_validated: false,
            dsn_validation_error: String::new(),
            app,
            state,
        }
    }
}

impl InputReceiver for ConnectionInputReceiver {
    fn receive_input(&mut self, event: Event) -> bool {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char(c) => {
                    if self.active {
                        self.input.push(c);
                        if self.name.is_empty() {
                            self.name_validated = self.input.len() > 0;
                        } else {
                            (self.dsn_validated, self.dsn_validation_error) =
                                validate_dsn(self.input.clone());
                        }
                        return true;
                    } else if c == 'n' {
                        let aw = self.state.read().unwrap().get_active_window();
                        if aw.id() == types::WindowTypeID::CONNECTIONS {
                            self.active = true;
                            return true;
                        }
                    }
                }
                KeyCode::Backspace => {
                    if self.active {
                        if !self.input.is_empty() {
                            self.input.pop();
                        }
                        return true;
                    }
                }
                KeyCode::Enter => {
                    if self.active {
                        if self.name.is_empty() {
                            if self.name_validated {
                                self.name = self.input.clone();
                                self.input.clear();
                            }
                        } else if !self.input.is_empty() {
                            if self.dsn_validated {
                                let dsn = self.input.clone();
                                let name = self.name.clone();
                                self.clear();
                                let mut data = self.app.persistent_data.write().unwrap();
                                data.add_connection(name, dsn);
                                self.state.write().unwrap().refresh();
                            }
                        }
                        return true;
                    }
                }

                KeyCode::Esc => {
                    self.clear();
                }
                _ => {}
            }
        }
        false
    }
    fn active(&self, event: Option<Event>) -> bool {
        self.active
    }
    fn clear(&mut self) {
        self.name.clear();
        self.input.clear();
        self.name_validated = false;
        self.dsn_validated = false;
        self.active = false;
    }

    fn render(&mut self, frame: &mut Frame, rect: Rect) {
        let lines;
        if self.name.is_empty() {
            let color = if self.name_validated {
                Color::Green
            } else {
                Color::Red
            };
            lines = vec![
                Line::from(Span::styled(
                    "Connection Name: ",
                    Style::default().fg(color),
                )),
                Line::from(Span::styled(
                    self.input.clone(),
                    Style::default().fg(Color::White),
                )),
            ];
        } else {
            let color = if self.dsn_validated {
                Color::Green
            } else {
                Color::Red
            };
            lines = vec![
                Line::from(Span::styled(
                    format!("DSN: {}", self.dsn_validation_error),
                    Style::default().fg(color),
                )),
                Line::from(Span::styled(
                    self.input.clone(),
                    Style::default().fg(Color::White),
                )),
            ];
        }
        let para = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title("New Connection"),
        );
        frame.render_widget(para, rect);
    }

    fn layout_size(&self) -> u16 {
        4
    }
}

struct DeleteConnectionInputReceiver {
    active: bool,
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
}

impl DeleteConnectionInputReceiver {
    pub fn new(
        app: Arc<Application>,
        state: Arc<RwLock<LayoutState>>,
    ) -> DeleteConnectionInputReceiver {
        DeleteConnectionInputReceiver {
            active: false,
            app,
            state,
        }
    }
}

impl InputReceiver for DeleteConnectionInputReceiver {
    fn receive_input(&mut self, event: Event) -> bool {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char(c) => {
                    if self.active {
                        if c == 'y' {
                            self.clear();
                            let mut data = self.app.persistent_data.write().unwrap();
                            let pos = self.state.read().unwrap().get_position();
                            data.connections.remove(pos as usize);
                            data.save();
                            self.state.write().unwrap().refresh();
                            return true;
                        } else if c == 'n' {
                            self.clear();
                            return true;
                        }
                    } else if c == 'd' {
                        let state = self.state.read().unwrap();
                        let aw = state.get_active_window();
                        let pos = state.get_position();
                        if aw.id() == types::WindowTypeID::CONNECTIONS && pos >= 0 {
                            self.active = true;
                            return true;
                        }
                    }
                }
                KeyCode::Esc => {
                    self.clear();
                }
                _ => {}
            }
        }
        false
    }
    fn active(&self, event: Option<Event>) -> bool {
        self.active
    }
    fn clear(&mut self) {
        self.active = false;
    }

    fn render(&mut self, frame: &mut Frame, rect: Rect) {
        let lines = vec![
            Line::from(Span::styled(
                "Are you sure you want to delete this connection?",
                Style::default().fg(Color::Red),
            )),
            Line::from(Span::styled(
                "Press 'y' to confirm, 'n' to cancel",
                Style::default().fg(Color::White),
            )),
        ];
        let para = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title("Delete Connection"),
        );
        frame.render_widget(para, rect);
    }

    fn layout_size(&self) -> u16 {
        4
    }
}

struct EditQueryInputReceiver<'a> {
    active: bool,
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
    textarea: TextArea<'a>,
}

impl<'a> EditQueryInputReceiver<'a> {
    pub fn new(
        app: Arc<Application>,
        state: Arc<RwLock<LayoutState>>,
    ) -> EditQueryInputReceiver<'a> {
        let ta = TextArea::default();
        EditQueryInputReceiver {
            active: false,
            app,
            state,
            textarea: ta,
        }
    }
}

impl<'a> InputReceiver for EditQueryInputReceiver<'a> {
    fn receive_input(&mut self, event: Event) -> bool {
        if self.active {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('r'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) = event
            {
                let mut state = self.state.write().unwrap();
                state.refresh();
                return true;
            }
            if let Event::Key(KeyEvent {
                code: KeyCode::Char('s'),
                modifiers: KeyModifiers::CONTROL,
                ..
            }) = event
            {
                let mut state = self.state.write().unwrap();
                state.refresh();
                self.active = false;
                self.textarea = TextArea::default();
                return true;
            }
        }
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char(c) => {
                    if self.active {
                    } else if c == 'e' {
                        let state = self.state.read().unwrap();
                        let aw = state.get_active_window();
                        if aw.id() == types::WindowTypeID::QUERY {
                            self.active = true;
                            for line in state.get_current_query().split("\n") {
                                self.textarea.insert_str(line);
                                self.textarea.insert_newline();
                            }
                            self.textarea.move_cursor(CursorMove::Top);
                            return true;
                        }
                    }
                }
                KeyCode::Esc => {
                    self.clear();
                    return true;
                }
                _ => {}
            }
        }
        if self.active {
            self.textarea.input(event);
            let mut state = self.state.write().unwrap();
            let lines: Vec<String> = self.textarea.clone().into_lines();
            state.update_custom_query(Some(lines.join("\n")));
            return true;
        }
        false
    }
    fn active(&self, event: Option<Event>) -> bool {
        self.active
    }
    fn clear(&mut self) {
        self.active = false;
        let mut state = self.state.write().unwrap();
        state.update_custom_query(None);
        self.textarea = TextArea::default();
    }

    fn render(&mut self, frame: &mut Frame, rect: Rect) {
        self.textarea.set_block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Edit Query"),
        );
        frame.render_widget(self.textarea.widget(), rect);
    }

    fn layout_size(&self) -> u16 {
        5
    }
}

struct ViewQueryInputReceiver {
    active: bool,
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
}

impl ViewQueryInputReceiver {
    pub fn new(app: Arc<Application>, state: Arc<RwLock<LayoutState>>) -> ViewQueryInputReceiver {
        ViewQueryInputReceiver {
            active: false,
            app,
            state,
        }
    }
}

impl<'a> InputReceiver for ViewQueryInputReceiver {
    fn receive_input(&mut self, event: Event) -> bool {
        false
    }
    fn active(&self, event: Option<Event>) -> bool {
        if event.is_some() {
            return false;
        }
        let state = self.state.read().unwrap();
        let aw = state.get_active_window();
        if aw.id() == types::WindowTypeID::QUERY {
            return true;
        }
        false
    }
    fn clear(&mut self) {
        //
    }

    fn render(&mut self, frame: &mut Frame, rect: Rect) {
        let state = self.state.read().unwrap();
        let text = state.get_current_query();
        let para = Paragraph::new(
            text.split("\n")
                .into_iter()
                .map(|s| Line::from(Span::styled(s, Style::default().fg(Color::Gray))))
                .collect::<Vec<_>>(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title("Query"),
        );
        frame.render_widget(para, rect);
    }

    fn layout_size(&self) -> u16 {
        5
    }
}

pub struct InputBar {
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
    input_receivers: Vec<Box<dyn InputReceiver>>,
}

impl InputBar {
    pub fn new(app: Arc<Application>, state: Arc<RwLock<LayoutState>>) -> InputBar {
        InputBar {
            input_receivers: vec![
                Box::new(CommandInputReceiver::new(
                    Arc::clone(&app),
                    Arc::clone(&state),
                )),
                Box::new(ConnectionInputReceiver::new(
                    Arc::clone(&app),
                    Arc::clone(&state),
                )),
                Box::new(DeleteConnectionInputReceiver::new(
                    Arc::clone(&app),
                    Arc::clone(&state),
                )),
                Box::new(EditQueryInputReceiver::new(
                    Arc::clone(&app),
                    Arc::clone(&state),
                )),
                Box::new(ViewQueryInputReceiver::new(
                    Arc::clone(&app),
                    Arc::clone(&state),
                )),
            ],
            app,
            state,
        }
    }

    fn handle_event(&mut self, event: Event) {
        for receiver in self.input_receivers.iter_mut() {
            if receiver.active(Some(event.clone())) {
                receiver.receive_input(event.clone());
                return;
            }
        }
        for receiver in self.input_receivers.iter_mut() {
            if receiver.receive_input(event.clone()) {
                return;
            }
        }
    }

    pub fn active(&self) -> bool {
        for receiver in self.input_receivers.iter() {
            if receiver.active(None) {
                return true;
            }
        }
        false
    }

    pub fn layout_size(&self) -> u16 {
        for receiver in self.input_receivers.iter() {
            if receiver.active(None) {
                return receiver.layout_size();
            }
        }
        3
    }

    pub fn render(&mut self, frame: &mut Frame, rect: Rect, event: Option<Event>) {
        if let Some(event) = event {
            self.handle_event(event);
        }

        for receiver in self.input_receivers.iter_mut() {
            if receiver.active(None) {
                receiver.render(frame, rect);
                return;
            }
        }

        let para = Paragraph::new("").block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title("Input"),
        );

        frame.render_widget(para, rect);
    }
}
