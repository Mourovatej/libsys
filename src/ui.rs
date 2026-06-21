use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};
use ratatui_textarea::TextArea;
pub enum Field {
    Title,
    Author,
    Tags,
    PublicationYear,
    Isbn,
    Location,
}
pub struct SearchForm<'a> {
    pub author: TextArea<'a>,
    pub title: TextArea<'a>,
    pub publication_year: TextArea<'a>,
    pub tags: TextArea<'a>,
    pub isbn: TextArea<'a>,
    pub location: TextArea<'a>,
    pub focused: Field,
}
impl SearchForm<'_> {
    pub fn focus_next(&mut self) {
        self.focused = match self.focused {
            Field::Title => Field::Author,
            Field::Author => Field::Tags,
            Field::Tags => Field::PublicationYear,
            Field::PublicationYear => Field::Isbn,
            Field::Isbn => Field::Location,
            Field::Location => Field::Title,
        }
    }
}
pub fn render_book_list(frame: &mut Frame, area: Rect) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(95), Constraint::Percentage(5)].as_ref())
        .split(area);
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(vertical_chunks[0]);
}
