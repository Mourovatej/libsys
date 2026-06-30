use std::path::Path;

use turso::Builder;

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
