use crate::app::App;
use anyhow::{Ok, Result};
use parser::*;

mod app;
mod models;
mod parser;
mod theme;
mod ui;

fn main() -> Result<()> {
    initialize_conf()?;
    let mut terminal = ratatui::init();
    let mut app = App::default();
    app.get_stuff()?;
    app.run(&mut terminal)?;

    ratatui::restore();
    Ok(())
}
