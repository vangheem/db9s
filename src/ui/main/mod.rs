use crate::app::Application;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::Constraint;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, Row, Table, TableState};
use ratatui::{layout::Rect, Frame};
use std::{sync::Arc, sync::RwLock};

use crate::ui::state::LayoutState;

use super::types;

pub struct MainArea {
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
}

impl MainArea {
    pub fn new(app: Arc<Application>, state: Arc<RwLock<LayoutState>>) -> MainArea {
        MainArea { app, state }
    }

    fn handle_event(&mut self, event: Event) {
        if let Event::Key(KeyEvent { code, .. }) = event {
            match code {
                KeyCode::Char('j') => {
                    let mut state = self.state.write().unwrap();
                    let current = state.get_position();
                    if current == -1 {
                        state.set_position(0);
                    } else {
                        state.set_position(current + 1);
                    }
                }
                KeyCode::Char('k') => {
                    let mut state = self.state.write().unwrap();
                    let current = state.get_position();
                    if current != -1 {
                        state.set_position(current - 1);
                    }
                }
                KeyCode::Char('r') => {
                    let mut state = self.state.write().unwrap();
                    state.refresh();
                }
                KeyCode::Char(' ') => {
                    // spacebar pressed
                    let mut state = self.state.write().unwrap();
                    state.select_current();
                }
                KeyCode::Enter => {
                    if self.state.read().unwrap().get_position() < 0 {
                        return;
                    }
                    self.state.write().unwrap().select_for_next_window();
                }
                _ => {}
                _ => {}
            }
        }
    }

    pub fn render(&mut self, frame: &mut Frame, rect: Rect, event: Option<Event>) {
        if let Some(event) = event.clone() {
            self.handle_event(event);
        }
        let wd = self.state.read().unwrap().get_window_data();
        let state = self.state.read().unwrap();

        let mut selected_rows = Vec::new();
        let window = state.get_active_window();
        if window.selection_type() == types::ItemSelectionType::MULTI {
            selected_rows = state
                .inner
                .read()
                .unwrap()
                .get_selection(window.id())
                .unwrap_or(Vec::new());
        }

        let rows: Vec<Row> = wd
            .rows
            .iter()
            .map(|dr| {
                if selected_rows.contains(&dr.id.clone()) {
                    return Row::new(
                        dr.data
                            .iter()
                            .map(|d| Cell::from(d.clone()).style(Style::default().fg(Color::Green)))
                            .collect::<Vec<Cell>>(),
                    );
                }
                Row::new(
                    dr.data
                        .iter()
                        .map(|d| Cell::from(d.clone()))
                        .collect::<Vec<Cell>>(),
                )
            })
            .collect();

        let mut table_state = TableState::default();

        let current = state.get_position();
        if current >= 0 && current < rows.len() as i32 {
            table_state.select(Some(current.clone() as usize));
        }

        let mut column_size = 100;
        if wd.columns.len() > 0 {
            column_size = (100 / wd.columns.len()) as u16;
        }
        let widths = wd
            .columns
            .iter()
            .map(|_| Constraint::Percentage(column_size))
            .collect::<Vec<_>>();

        let table = Table::new(rows)
            .widths(&widths)
            .header(Row::new(wd.columns.clone()).style(Style::default().fg(Color::Yellow)))
            .block(Block::default().title(window.title()).borders(Borders::ALL))
            .column_spacing(1)
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().bg(Color::Gray));

        frame.render_stateful_widget(table, rect, &mut table_state);
    }
}
