use tokio_postgres::{Error, NoTls};
pub async fn test_query() -> Result<Vec<tokio_postgres::Row>, Error> {
    let (client, connection) =
        tokio_postgres::connect("host=localhost dbname=libsys user=libsys_user", NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    let rows = client.query("SELECT id,title FROM library", &[]).await?;
    Ok(rows)
}
