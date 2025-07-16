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
use ratatui::widgets::ListState;
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{text::Text, Frame};
use ratatui::{DefaultTerminal, Terminal};
use std::char;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::vec;

pub struct App {
    pub mode: Mode,
    pub selection_state: ListState,
    pub selected_interface: InterfaceSelected,
    pub prev_stats: Option<Vec<NetworkStats>>,
    pub rx_data: Vec<(f64, f64)>,
    pub tx_data: Vec<(f64, f64)>,
    pub start_time: Instant,
    pub last_sample_time: f64,
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
            selection_state: ListState::default(),
            mode: Mode::Normal,
            selected_interface: InterfaceSelected::All,
            prev_stats: None,
            rx_data: Vec::new(),
            tx_data: Vec::new(),
            start_time: Instant::now(),
            last_sample_time: 0.0,
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

pub enum Mode {
    Normal,
    SelectingInterface { filter: String, index: usize },
}

#[derive(Clone)]
pub enum InterfaceSelected {
    All,
    Interface(String),
}

#[derive(Default, Clone, Copy)]
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
        let interface_name_vec: Vec<String> = parse_proc_net_dev()?
            .iter()
            .map(|s| s.name.clone())
            .collect();

        loop {
            let now = self.start_time.elapsed().as_secs_f64();
            self.window = [now - 5.0, now];
            let vec_stats = parse_proc_net_dev()?;

            let _ = terminal.draw(|frame| self.render(frame, &vec_stats));

            if let Some(prev_data) = &self.prev_stats {
                for (prev, new) in prev_data.iter().zip(vec_stats.iter()) {
                    let rx_delta = new.receive.bytes.saturating_sub(prev.receive.bytes);
                    let tx_delta = new.transmit.bytes.saturating_sub(prev.transmit.bytes);

                    self.rx_data.push((now, rx_delta as f64));
                    self.tx_data.push((now, tx_delta as f64));
                }
            }
            self.prev_stats = Some(vec_stats);

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
                    KeyCode::Char('i') => self.current_tab = Tab::Interface,
                    KeyCode::Char('t') => self.current_tab = Tab::Tcp,
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
