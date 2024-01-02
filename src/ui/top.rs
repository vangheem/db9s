use crate::app::Application;
use crossterm::event::Event;
use ratatui::prelude::{Constraint, Line};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Row, Table},
    Frame,
};
use std::collections::HashMap;
use std::{sync::Arc, sync::RwLock};

use crate::ui::state::LayoutState;
use ratatui::layout::{Direction, Layout};

use super::types;

pub struct TopArea {
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
}

impl TopArea {
    pub fn new(app: Arc<Application>, state: Arc<RwLock<LayoutState>>) -> TopArea {
        TopArea { app, state }
    }

    pub fn render(&mut self, frame: &mut Frame, rect: Rect, event: Option<Event>) {
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

        let shortcuts = vec![
            ("j", "Down"),
            ("k", "Up"),
            ("r", "Refresh"),
            ("space", "Select"),
            ("enter", "Open"),
            (":", "Command"),
            ("n", "New"),
            ("d", "Delete"),
            ("e", "Edit"),
            ("Esc", "Cancel"),
            ("r", "Refresh"),
            ("Control+r", "Refresh"),
            ("Control+s", "Save"),
        ];

        let shortcuts_table = Table::new(
            shortcuts
                .iter()
                .map(|(k, v)| Row::new(vec![Text::raw(k.to_string()), Text::raw(v.to_string())]))
                .collect::<Vec<_>>(),
        )
        .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)]);

        let commands = vec![
            "connections",
            "databases",
            "tables",
            "schemas",
            "columns",
            "query",
        ];
        let commands_table = Table::new(
            commands
                .iter()
                .map(|c| Row::new(vec![Text::raw(c.to_string())]))
                .collect::<Vec<_>>(),
        )
        .widths(&[Constraint::Percentage(100)]);

        let table = Table::new(vec![Row::new(values)])
            .widths(&[Constraint::Percentage(50), Constraint::Percentage(50)])
            .header(Row::new(headers).style(Style::default().fg(Color::Yellow)))
            .column_spacing(1)
            .style(Style::default().fg(Color::White));

        let areas = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(40),
                Constraint::Percentage(40),
                Constraint::Percentage(20),
            ])
            .split(rect);

        frame.render_widget(table, areas[0]);
        frame.render_widget(shortcuts_table, areas[1]);
        frame.render_widget(commands_table, areas[2]);
    }
}
