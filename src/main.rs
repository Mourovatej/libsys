use tokio_postgres::{Error, NoTls};
mod db;
mod ui;

#[tokio::main]
async fn main() -> Result<(), Error> {
    db::test_query().await?;
    Ok(())
}
