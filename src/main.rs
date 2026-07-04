use std::error::Error;

mod app;
mod db;
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let path = {
        let mut p = dirs::home_dir().expect("could not find home directory");
        p.push(".config/libsys/library.db");
        p
    };

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut app = app::App::new(path.to_str().unwrap()).await?;
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal).await;
    ratatui::restore();
    result
}
