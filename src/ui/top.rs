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

use super::types::SelectionType;

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
        let row_column_pairs = vec![
            ("Connection", SelectionType::Connection),
            ("Database", SelectionType::Database),
            ("Schema", SelectionType::Schema),
            ("Table", SelectionType::Table),
            ("Columns", SelectionType::Column),
        ];
        let mut headers = Vec::new();
        let mut values = Vec::new();
        for (name, selection_type) in row_column_pairs {
            let value = inner.get_selection(selection_type);
            if value.is_none() {
                continue;
            }
            headers.push(name);
            values.push(value.unwrap().join(","));
        }

        let len = values.len();
        let mut width = 100;
        if len > 0 {
            width = 100 / len;
        }

        let table = Table::new(
            vec![Row::new(values)],
            vec![Constraint::Percentage(width as u16); len],
        )
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
