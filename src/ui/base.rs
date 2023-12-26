use crossterm::event::Event;
use ratatui::{layout::Rect, Frame};

trait ListArea {
    fn render(&mut self, frame: &mut Frame, rect: Rect, event: Option<Event>);
    fn get_key(&self) -> &'static str;
}
