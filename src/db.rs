use std::path::Path;

use tokio_postgres::{Error, NoTls};
use turso::Builder;
pub async fn test_query() -> Result<Vec<tokio_postgres::Row>, Error> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost dbname=libsys user=libsys_user", NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let rows = client.query("SELECT * FROM library", &[]).await?;
    Ok(rows)
}

pub async fn create_or_open_db(path: &str) -> turso::Result<(turso::Database, turso::Connection)> {
    let is_new = !Path::new(path).exists();

    let db = Builder::new_local(path).build().await?;
    let conn = db.connect()?;
    conn.execute(
        r#"CREATE TABLE IF NOT EXISTS library (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    author TEXT,
    title TEXT,
    publication_year INTEGER,
    tags TEXT,
    return_date TEXT,
    location TEXT,
    isbn TEXT,
    notes TEXT
    
    ) "#,
        (),
    )
    .await?;

    if is_new {
        setup_fts(&conn).await?;
    }
    Ok((db, conn))
}

async fn setup_fts(conn: &turso::Connection) -> turso::Result<()> {
    conn.execute(
        r#"CREATE INDEX idx_library_fts ON library
           USING fts (title, author, tags, notes, location, isbn)"#,
        (),
    )
    .await?;
    Ok(())
}
