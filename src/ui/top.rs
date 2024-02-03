use crate::app::Application;
use chrono::format;
use crossterm::event::Event;
use ratatui::prelude::Constraint;
use ratatui::text::{Line, Span};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Row, Table},
    Frame,
};
use std::{sync::Arc, sync::RwLock};

use crate::ui::state::LayoutState;
use ratatui::layout::{Alignment, Direction, Layout};

use super::types;

pub struct TopArea {
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
}

static SHORTCUTS: [(&str, &str); 14] = [
    ("j", "Down"),
    ("k", "Up"),
    ("Control-j", "Bottom"),
    ("Control-k", "Top"),
    ("r", "Refresh"),
    ("space", "Select"),
    ("enter", "Open"),
    (":", "Command"),
    ("n", "New"),
    ("d", "Delete"),
    ("e", "Edit"),
    ("Esc", "Cancel"),
    ("Control+r", "Refresh"),
    ("Control+s", "Save"),
];
static COMMNDS: [&str; 7] = [
    "connections",
    "databases",
    "tables",
    "schemas",
    "columns",
    "query",
    "history",
];

impl TopArea {
    pub fn new(app: Arc<Application>, state: Arc<RwLock<LayoutState>>) -> TopArea {
        TopArea { app, state }
    }

    fn get_shortcuts(&self, split: usize, part: usize) -> Table {
        // let current_window = self.state.read().unwrap().get_active_window();
        let mut take = (SHORTCUTS.len() / split) + 1;
        let skip = take * part;
        if part == split - 1 {
            take += SHORTCUTS.len() % split;
        }
        Table::new(
            SHORTCUTS
                .iter()
                .skip(skip)
                .take(take)
                .map(|(k, v)| {
                    Row::new(vec![
                        Line::from(Span::styled(
                            format!("<{}>: ", k),
                            Style::default().fg(Color::Magenta),
                        ))
                        .alignment(Alignment::Right),
                        Line::from(Span::styled(
                            format!("{}", v),
                            Style::default().fg(Color::Gray),
                        )),
                    ])
                })
                .collect::<Vec<_>>(),
        )
        .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)])
    }

    fn get_commands(&self, split: usize, part: usize) -> Table {
        let mut take = COMMNDS.len() / split;
        let skip = take * part;
        if part == split - 1 {
            take += COMMNDS.len() % split;
        }
        Table::new(
            COMMNDS
                .iter()
                .skip(skip)
                .take(take)
                .map(|c| {
                    Row::new(vec![Line::from(vec![
                        Span::styled(format!("["), Style::default().fg(Color::Yellow)),
                        Span::styled(format!("{}", c), Style::default().fg(Color::Green)),
                        Span::styled(format!("]"), Style::default().fg(Color::Yellow)),
                    ])])
                })
                .collect::<Vec<_>>(),
        )
        .widths(&[Constraint::Percentage(100)])
    }

    fn get_selection_table(&self) -> Table {
        let state = self.state.read().unwrap();
        let inner = state.inner.read().unwrap();
        let mut headers = Vec::new();
        let mut values = Vec::new();
        for window_type in types::WINDOW_TYPES {
            let value = inner.get_selection(window_type.id());
            if value.is_none() {
                continue;
            }
            let mut value = value.unwrap();
            if value.is_empty() {
                continue;
            }
            if window_type.id() == types::WindowTypeID::CONNECTIONS {
                let conns = state.get_connections();
                let conn = conns.iter().find(|c| c.id == value[0]);
                if conn.is_some() {
                    value = vec![conn.unwrap().name.clone()];
                }
            }
            headers.push(window_type.title());
            values.push(value.join(","));
        }

        Table::new(
            headers
                .iter()
                .zip(values.iter())
                .map(|(h, v)| {
                    Row::new(vec![
                        Line::from(Span::styled(
                            format!("{}: ", h),
                            Style::default().fg(Color::Yellow),
                        ))
                        .alignment(Alignment::Right),
                        Line::from(Span::styled(
                            v.to_string(),
                            Style::default().fg(Color::White),
                        )),
                    ])
                })
                .collect::<Vec<_>>(),
        )
        .widths(&[Constraint::Percentage(30), Constraint::Percentage(70)])
        .column_spacing(1)
        .style(Style::default().fg(Color::White))
    }

    pub fn render(&mut self, frame: &mut Frame, rect: Rect, event: Option<Event>) {
        let shortcuts_table1 = self.get_shortcuts(3, 0);
        let shortcuts_table2 = self.get_shortcuts(3, 1);
        let shortcuts_table3 = self.get_shortcuts(3, 2);

        let commands_table1 = self.get_commands(2, 0);
        let commands_table2 = self.get_commands(2, 1);

        let selection_table = self.get_selection_table();

        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Percentage(5),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
                Constraint::Percentage(15),
            ])
            .split(rect);

        frame.render_widget(selection_table, areas[0]);
        frame.render_widget(commands_table1, areas[1]);
        frame.render_widget(commands_table2, areas[2]);
        // leave out spacer
        frame.render_widget(shortcuts_table1, areas[4]);
        frame.render_widget(shortcuts_table2, areas[5]);
        frame.render_widget(shortcuts_table3, areas[6]);
    }
}
