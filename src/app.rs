use crossterm::event::poll;
use crossterm::event::{self, Event, KeyCode};
use ratatui::style::Modifier;
use ratatui::widgets::{Borders, Row, Table, TableState};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Clear},
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

pub fn render_book_list(frame: &mut Frame, items: &mut [Book], table_state: &mut TableState) {
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(99), Constraint::Percentage(1)].as_ref())
        .split(frame.area());

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
    let table_block = Block::default().borders(Borders::ALL);
    let table = Table::new(
        rows,
        [
            Constraint::Length(5),
            Constraint::Percentage(17),
            Constraint::Percentage(17),
            Constraint::Length(10),
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Length(15),
            Constraint::Length(30),
            Constraint::Fill(1),
        ],
    )
    .block(table_block)
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
    frame.render_stateful_widget(table, vertical_chunks[0], table_state);
}

pub fn search_popup(frame: &mut Frame, search_line: &mut TextArea) {
    let popup_block = Block::bordered().title("Search");
    let centered_area = frame
        .area()
        .centered(Constraint::Percentage(50), Constraint::Length(3));
    search_line.set_block(popup_block);
    frame.render_widget(Clear, centered_area);
    frame.render_widget(&*search_line, centered_area);
}

pub struct App<'a> {
    search_line: TextArea<'a>,
    search_popup: bool,
    items: Vec<Book>,
    item_table_state: TableState,
    should_quit: bool,
    db: turso::Database,
    conn: turso::Connection,
}

impl App<'_> {
    pub async fn new(path: &str) -> turso::Result<Self> {
        let (db, conn) = db::create_or_open_db(path).await?;
        Ok(Self {
            search_line: TextArea::default(),
            search_popup: false,
            item_table_state: TableState::default(),
            items: Vec::new(),
            should_quit: false,
            db,
            conn,
        })
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Box<dyn Error>> {
        //App::insert_sample_data(&self.conn).await?;
        let result = Self::query_whole(&self.conn).await?;
        self.items = Self::parse_result(result).await?;
        self.item_table_state.select_first();
        while !self.should_quit {
            terminal.draw(|frame| {
                render_book_list(frame, &mut self.items, &mut self.item_table_state);
                if self.search_popup {
                    search_popup(frame, &mut self.search_line);
                }
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
                        render_book_list(frame, &mut self.items, &mut self.item_table_state);
                    })?;
                }
                Event::Key(key) => {
                    if self.search_popup {
                        match key.code {
                            KeyCode::Enter => {
                                let result = App::search(
                                    &self.conn,
                                    &self.search_line.lines().join(" ").to_string(),
                                )
                                .await?;
                                self.items = App::parse_result(result).await?;
                                self.item_table_state.select_first();
                                self.search_popup = false;
                            }
                            _ => {
                                self.search_line.input(key);
                            }
                        }
                    }

                    match key.code {
                        KeyCode::Esc => self.should_quit = true,

                        KeyCode::Up => self.item_table_state.select_previous(),
                        KeyCode::Down => self.item_table_state.select_next(),
                        KeyCode::Char('/') => {
                            if self.search_popup {
                                self.search_line.clear();
                            }
                            self.search_popup = !self.search_popup;
                        }
                        KeyCode::Char('r') => {
                            let result = App::query_whole(&self.conn).await?;
                            self.items = App::parse_result(result).await?;
                        }
                        _ => {}
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
