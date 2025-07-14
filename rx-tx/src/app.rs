use crate::models::*;
use crate::parser::*;
use crate::ui::*;
use anyhow::{anyhow, Error, Ok, Result};
use clap::builder::Str;
use crossterm::event::{self, read, Event, KeyCode};
use ratatui::layout::{Alignment, Constraint, Layout, Margin};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::scrollbar;
use ratatui::text::{Line, Masked, Span};
use ratatui::widgets::block::title;
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{text::Text, Frame};
use ratatui::{DefaultTerminal, Terminal};
use std::collections::HashMap;
use std::time::{Duration, Instant};

pub struct App {
    pub prev_stats: Option<Vec<NetworkStats>>,
    pub graph_data: HashMap<String, Vec<(f64, f64)>>,
    pub start_time: Instant,
    pub window: [f64; 2],
    pub raw_bytes: bool,
    pub byte_unit: ByteUnit,
    pub current_tab: Tab,
    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    pub horizontal_scroll: usize,
    pub vertical_scroll: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            prev_stats: None,
            graph_data: HashMap::new(),
            start_time: Instant::now(),
            window: [0.0, 60.0],
            raw_bytes: false,
            byte_unit: ByteUnit::default(),
            current_tab: Tab::default(),
            vertical_scroll_state: ScrollbarState::new(0),
            horizontal_scroll_state: ScrollbarState::new(0),
            horizontal_scroll: 0,
            vertical_scroll: 0,
        }
    }
}

#[derive(Default)]
pub enum ByteUnit {
    #[default]
    Binary,
    Decimal,
}

#[derive(Default, Copy, Clone)]
pub enum Tab {
    #[default]
    Interface,
    Tcp,
}

impl Tab {
    pub fn titles() -> Vec<&'static str> {
        vec!["Interface Mode (I)", "TCP Mode (T)"]
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            let vec_stats = parse_proc_net_dev()?;
            let _ = terminal.draw(|frame| self.render(frame, &vec_stats));
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if !event::poll(timeout)? {
                last_tick = Instant::now();
                continue;
            }

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Char('h') => self.scroll_left(),
                    KeyCode::Char('j') => self.scroll_down(),
                    KeyCode::Char('k') => self.scroll_up(),
                    KeyCode::Char('l') => self.scroll_right(),
                    KeyCode::Char('r') => self.raw_bytes = !self.raw_bytes,
                    KeyCode::Char('d') => self.byte_unit = ByteUnit::Decimal,
                    KeyCode::Char('b') => self.byte_unit = ByteUnit::Binary,
                    _ => {}
                }
            }
        }
    }

    pub fn scroll_up(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
        self.update_scroll_state();
    }

    pub fn scroll_down(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_add(1);
        self.update_scroll_state();
    }

    pub fn scroll_left(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_sub(1);
        self.update_scroll_state();
    }

    pub fn scroll_right(&mut self) {
        self.horizontal_scroll = self.horizontal_scroll.saturating_add(1);
        self.update_scroll_state();
    }
    pub fn update_scroll_state(&mut self) {
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .position(self.horizontal_scroll);
    }
    pub fn render(&mut self, frame: &mut Frame, data: &Vec<NetworkStats>) {
        crate::ui::draw_interface_mode(self, frame, data);
    }
}
