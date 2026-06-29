use crossterm::event::poll;
use crossterm::event::{self, Event, KeyCode};
use ratatui::style::Modifier;
use ratatui::widgets::{Row, Table, TableState};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders},
};
use ratatui_textarea::TextArea;
use std::error::Error;
use std::time::Duration;

use crate::db;

pub struct Book {
    pub id: u32,
    pub author: Option<String>,
    pub title: Option<String>,
    pub publication_year: Option<u32>,
    pub return_date: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub isbn: Option<String>,
    pub tags: Option<String>,
}
#[derive(PartialEq, Default, Clone, Copy)]
pub enum Field {
    #[default]
    Title,
    Author,
    Tags,
    PublicationYear,
    Isbn,
    Location,
}

pub enum Screen {
    Search,
    Table,
}

#[derive(Default)]
pub struct SearchForm<'a> {
    pub author: TextArea<'a>,
    pub title: TextArea<'a>,
    pub publication_year: TextArea<'a>,
    pub tags: TextArea<'a>,
    pub isbn: TextArea<'a>,
    pub location: TextArea<'a>,
    pub focused: Field,
    pub dimmed: bool,
}

impl<'a> SearchForm<'a> {
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
    pub fn focus_previous(&mut self) {
        self.focused = match self.focused {
            Field::Title => Field::Location,
            Field::Location => Field::Isbn,
            Field::Isbn => Field::PublicationYear,
            Field::PublicationYear => Field::Tags,
            Field::Tags => Field::Author,
            Field::Author => Field::Title,
        }
    }

    pub fn focused_textarea_mut(&mut self) -> &mut TextArea<'a> {
        match self.focused {
            Field::Title => &mut self.title,
            Field::Author => &mut self.author,
            Field::Tags => &mut self.tags,
            Field::PublicationYear => &mut self.publication_year,
            Field::Isbn => &mut self.isbn,
            Field::Location => &mut self.location,
        }
    }
}

pub fn render_book_list(
    frame: &mut Frame,
    search_form: &mut SearchForm,
    items: &mut [Book],
    table_state: &mut TableState,
) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(95), Constraint::Percentage(5)].as_ref())
        .split(frame.area());

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(vertical_chunks[0]);

    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
            Constraint::Ratio(1, 6),
        ])
        .split(main_chunks[0]);

    let fields: [(Field, &mut TextArea, &str); 6] = [
        (Field::Title, &mut search_form.title, "Title"),
        (Field::Author, &mut search_form.author, "Author"),
        (Field::Tags, &mut search_form.tags, "Tags"),
        (
            Field::PublicationYear,
            &mut search_form.publication_year,
            "Publication Year",
        ),
        (Field::Isbn, &mut search_form.isbn, "ISBN"),
        (Field::Location, &mut search_form.location, "Location"),
    ];

    let focused = search_form.focused;

    for (i, (field, textarea, label)) in fields.into_iter().enumerate() {
        let is_focused = field == focused;

        let block = Block::default().borders(Borders::ALL).title(label).style(
            match (is_focused, search_form.dimmed) {
                (true, true) => Style::default().add_modifier(Modifier::DIM),
                (true, false) => Style::default().fg(ratatui::style::Color::Green),
                (false, true) => Style::default().add_modifier(Modifier::DIM),
                (false, false) => Style::default(),
            },
        );

        textarea.set_block(block);
        frame.render_widget(&*textarea, form_chunks[i]);

        let rows: Vec<Row> = items
            .iter()
            .map(|book| {
                Row::new(vec![
                    book.id.clone().to_string(),
                    book.title.clone().unwrap_or_default(),
                    book.author.clone().unwrap_or_default(),
                    book.publication_year.unwrap_or_default().to_string(),
                    book.location.clone().unwrap_or_default(),
                    book.return_date.clone().unwrap_or_default(),
                    book.isbn.clone().unwrap_or_default(),
                    book.notes.clone().unwrap_or_default(),
                    book.tags.clone().unwrap_or_default(),
                ])
            })
            .collect();
        let table = Table::new(
            rows,
            [
                Constraint::Percentage(5),
                Constraint::Percentage(17),
                Constraint::Percentage(17),
                Constraint::Percentage(5),
                Constraint::Percentage(11),
                Constraint::Percentage(11),
                Constraint::Percentage(11),
                Constraint::Percentage(11),
                Constraint::Percentage(11),
            ],
        )
        .header(
            Row::new(vec![
                "id",
                "Title",
                "Author",
                "Pub Year",
                "Location",
                "Return Date",
                "ISBN",
                "Notes",
                "Tags",
            ])
            .style(Style::default().add_modifier(Modifier::UNDERLINED | Modifier::BOLD)),
        )
        .row_highlight_style(Style::default().bold())
        .highlight_symbol("> ");
        frame.render_stateful_widget(table, main_chunks[1], table_state);
    }
}

pub struct App<'a> {
    search_form: SearchForm<'a>,
    items: Vec<Book>,
    item_table_state: TableState,
    should_quit: bool,
    focused_screen: Screen,
    db: turso::Database,
    conn: turso::Connection,
}

impl App<'_> {
    pub async fn new(path: &str) -> turso::Result<Self> {
        let (db, conn) = db::create_or_open_db(path).await?;
        Ok(Self {
            search_form: SearchForm::default(),
            item_table_state: TableState::default(),
            items: Vec::new(),
            should_quit: false,
            focused_screen: Screen::Search,
            db,
            conn,
        })
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
        App::insert_sample_data(&self.conn).await?;
        let result = Self::query_whole(&self.conn).await?;
        self.items = Self::parse_result(result).await?;
        while !self.should_quit {
            terminal.draw(|frame| {
                render_book_list(
                    frame,
                    &mut self.search_form,
                    &mut self.items,
                    &mut self.item_table_state,
                );
            })?;
            self.handle_events(terminal).await?;
        }
        Ok(())
    }

    pub async fn handle_events(
        &mut self,
        terminal: &mut DefaultTerminal,
    ) -> Result<(), Box<dyn Error>> {
        if poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Resize(_, _) => {
                    terminal.draw(|frame| {
                        render_book_list(
                            frame,
                            &mut self.search_form,
                            &mut self.items,
                            &mut self.item_table_state,
                        );
                    })?;
                }
                Event::Key(key) => {
                    match key.code {
                        KeyCode::Esc => self.should_quit = true,
                        _ => {}
                    }
                    match self.focused_screen {
                        Screen::Search => match key.code {
                            KeyCode::Up => self.search_form.focus_previous(),
                            KeyCode::Down => self.search_form.focus_next(),
                            KeyCode::Tab => {
                                self.focused_screen = Screen::Table;
                                self.search_form.dimmed = true;
                            }
                            _ => {
                                self.search_form.focused_textarea_mut().input(key);
                            }
                        },
                        Screen::Table => match key.code {
                            KeyCode::Up => self.item_table_state.select_previous(),
                            KeyCode::Down => self.item_table_state.select_next(),
                            KeyCode::Tab => {
                                self.focused_screen = Screen::Search;
                                self.search_form.dimmed = false;
                            }
                            KeyCode::Char('/') => {
                                let result = App::search(&self.conn, "Lee").await?;
                                self.items = App::parse_result(result).await?;
                            }

                            _ => {}
                        },
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub async fn insert_sample_data(conn: &turso::Connection) -> turso::Result<()> {
        let books = vec![
            (
                "George Orwell",
                "1984",
                1949,
                "dystopian,classic,fiction",
                "",
                "Shelf A1",
                "9780451524935",
                "Re-read for book club",
            ),
            (
                "Harper Lee",
                "To Kill a Mockingbird",
                1960,
                "classic,fiction,drama",
                "",
                "Shelf A1",
                "9780061120084",
                "",
            ),
            (
                "J.R.R. Tolkien",
                "The Hobbit",
                1937,
                "fantasy,adventure",
                "2024-11-02",
                "Shelf B3",
                "9780547928227",
                "Lent to Sam",
            ),
            (
                "Frank Herbert",
                "Dune",
                1965,
                "sci-fi,classic",
                "",
                "Shelf B1",
                "9780441172719",
                "",
            ),
            (
                "Mary Shelley",
                "Frankenstein",
                1818,
                "gothic,classic,horror",
                "",
                "Shelf A2",
                "9780486282114",
                "",
            ),
        ];

        for (author, title, year, tags, return_date, location, isbn, notes) in books {
            conn.execute(
            r#"INSERT INTO library (author, title, publication_year, tags, return_date, location, isbn, notes)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)"#,
            (author, title, year, tags, return_date, location, isbn, notes),
        )
        .await?;
        }

        Ok(())
    }
    pub async fn query_whole(conn: &turso::Connection) -> turso::Result<turso::Rows> {
        conn.query("SELECT * FROM library", ()).await
    }
    pub async fn search(conn: &turso::Connection, query: &str) -> turso::Result<turso::Rows> {
        conn.query(
            r#"SELECT *, fts_score(title, author, tags, notes, location, isbn, ?1) AS score
           FROM library
           WHERE fts_match(title, author, tags, notes, location, isbn, ?1)
           ORDER BY score ASC"#,
            (query,),
        )
        .await
    }
    pub async fn parse_result(mut rows: turso::Rows) -> turso::Result<Vec<Book>> {
        let mut books: Vec<Book> = Vec::new();
        while let Some(row) = rows.next().await? {
            books.push(Book {
                id: row.get::<u32>(0)?,
                author: row.get::<Option<String>>(1)?,
                title: row.get::<Option<String>>(2)?,
                publication_year: row.get::<Option<u32>>(3)?,
                tags: row.get::<Option<String>>(4)?,
                return_date: row.get::<Option<String>>(5)?,
                location: row.get::<Option<String>>(6)?,
                isbn: row.get::<Option<String>>(7)?,
                notes: row.get::<Option<String>>(8)?,
            });
        }
        Ok(books)
    }
}
