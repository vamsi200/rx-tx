#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use crate::app::App;
use anyhow::{anyhow, Error, Ok, Result};
use models::*;
use parser::*;

mod app;
mod models;
mod parser;
mod ui;

fn main() -> Result<()> {
    initialize_conf()?;
    let mut terminal = ratatui::init();
    let mut app = App::default();
    app.get_stuff(&mut terminal)?;
    app.run(&mut terminal)?;

    ratatui::restore();
    Ok(())
}
