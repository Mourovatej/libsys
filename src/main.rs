use std::error::Error;
use tokio_postgres::NoTls;
mod app;
mod db;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = app::App::new("library.db").await?;
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal).await;
    ratatui::restore();
    result
}
