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

        let len = values.len();
        let mut width = 100;
        if len > 0 {
            width = 100 / len;
        }

        let widths = vec![Constraint::Percentage(width as u16); len];
        let table = Table::new(vec![Row::new(values)])
            .widths(&widths)
            .header(Row::new(headers).style(Style::default().fg(Color::Yellow)))
            .column_spacing(1)
            .style(Style::default().fg(Color::White));

        let areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(2), Constraint::Min(2)])
            .split(rect);

        frame.render_widget(table, areas[0]);
        frame.render_widget(
            Paragraph::new(vec![
                Line::from("Keys: j, k, space, enter, :".to_string()),
                Line::from(
                    "Command: connections, databases, tables, schemas, columns, query".to_string(),
                ),
            ]),
            areas[1],
        );
    }
}
