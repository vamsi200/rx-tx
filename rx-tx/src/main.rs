#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use anyhow::{anyhow, Error, Ok, Result};
use app::App;
use models::*;
use parser::*;
mod app;
mod models;
mod parser;
mod ui;

fn main() -> Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::default();
    app.run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}
