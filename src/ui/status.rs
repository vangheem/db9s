use crate::app::Application;
use crossterm::event::Event;
use ratatui::style::Color;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Block, Borders};
use ratatui::{
    layout::Rect,
    style::Style,
    Frame,
};
use std::{sync::Arc, sync::RwLock};

use crate::ui::state::LayoutState;

pub struct StatusArea {
    app: Arc<Application>,
    state: Arc<RwLock<LayoutState>>,
}

impl StatusArea {
    pub fn new(app: Arc<Application>, state: Arc<RwLock<LayoutState>>) -> StatusArea {
        StatusArea { app, state }
    }

    fn get_error(&self) -> Option<String> {
        let state = self.state.read().unwrap();
        let inner = state.inner.read().unwrap();
        inner.error.clone()
    }

    pub fn layout_size(&self) -> u16 {
        if self.get_error().is_some() {
            3
        } else {
            0
        }
    }

    pub fn render(&mut self, frame: &mut Frame, rect: Rect, event: Option<Event>) {
        let error = self.get_error();
        if error.is_none() {
            return;
        }
        let lines = vec![Line::from(Span::styled(
            error.unwrap(),
            Style::default().fg(Color::Green),
        ))];
        let para = Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default())
                .title("Error"),
        );
        frame.render_widget(para, rect);
    }
}
