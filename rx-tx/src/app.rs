use crate::models::*;
use crate::parser::*;
use crate::ui::*;
use anyhow::{anyhow, Error, Result};
use clap::builder::Str;
use crossterm::event::{self, read, Event, KeyCode};
use crossterm::style::SetStyle;
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
use std::result::Result::Ok;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::vec;

#[derive(Clone)]
pub struct App {
    pub is_full_screen: bool,
    pub enter_tick_active: bool,
    pub interface_name: String,
    pub tick_rate: Duration,
    pub tick_value: String,
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
    pub selected_tab: Tab,
    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    pub horizontal_scroll: usize,
    pub vertical_scroll: usize,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tick_value: String::new(),
            enter_tick_active: false,
            tick_rate: Duration::from_millis(250),
            is_full_screen: false,
            interface_name: String::new(),
            selection_state: {
                let mut state = ListState::default();
                state.select(Some(0));
                state
            },
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
            selected_tab: Tab::default(),
            vertical_scroll_state: ScrollbarState::new(0),
            horizontal_scroll_state: ScrollbarState::new(0),
            horizontal_scroll: 0,
            vertical_scroll: 0,
        }
    }
}

#[derive(Default, Clone)]
pub enum ByteUnit {
    #[default]
    Binary,
    Decimal,
}

#[derive(Clone)]
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
        vec!["Net/Dev (I)", "Net/Tcp (T)"]
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut last_tick = Instant::now();
        let interface_name_vec: Vec<String> = parse_proc_net_dev()?
            .iter()
            .map(|s| s.name.clone())
            .collect();

        let new_len = interface_name_vec.len();
        self.vertical_scroll_state = self.vertical_scroll_state.content_length(new_len);
        self.horizontal_scroll_state = self.horizontal_scroll_state.content_length(new_len);

        loop {
            let now = self.start_time.elapsed().as_secs_f64();
            self.window = [now - 5.0, now];
            let vec_stats = parse_proc_net_dev()?;
            let _ = terminal.draw(|frame| self.render(frame, &vec_stats));

            if let Some(prev_data) = &self.prev_stats {
                for (prev, new) in prev_data.iter().zip(vec_stats.iter()) {
                    let rx_delta = new.receive.bytes.saturating_sub(prev.receive.bytes);
                    let tx_delta = new.transmit.bytes.saturating_sub(prev.transmit.bytes);

                    match &mut self.selected_interface {
                        InterfaceSelected::All => {
                            self.interface_name = String::from("All");
                            self.rx_data.push((now, rx_delta as f64));
                            self.tx_data.push((now, tx_delta as f64));
                        }
                        InterfaceSelected::Interface(s) => {
                            if new.name == s.to_string() {
                                self.interface_name = s.to_string();
                                self.rx_data.push((now, rx_delta as f64));
                                self.tx_data.push((now, tx_delta as f64));
                            }
                        }
                    }
                }
            }
            self.prev_stats = Some(vec_stats);

            let timeout = self.tick_rate.saturating_sub(last_tick.elapsed());
            if !event::poll(timeout)? {
                last_tick = Instant::now();
                continue;
            }

            if let Event::Key(key) = event::read()? {
                match &mut self.mode {
                    Mode::Normal => match key.code {
                        KeyCode::Char('f') => {
                            self.mode = Mode::SelectingInterface {
                                filter: String::new(),
                                index: 0,
                            }
                        }
                        KeyCode::Char('k') => {
                            self.enter_tick_active = true;
                            self.tick_value.clear();
                            continue;
                        }

                        KeyCode::Char('e') => self.is_full_screen = !self.is_full_screen,
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Left => self.scroll_left(),
                        KeyCode::Down => self.scroll_down(),
                        KeyCode::Up => self.scroll_up(),
                        KeyCode::Right => self.scroll_right(),
                        KeyCode::Char('r') => self.raw_bytes = !self.raw_bytes,
                        KeyCode::Char('d') => self.byte_unit = ByteUnit::Decimal,
                        KeyCode::Char('b') => self.byte_unit = ByteUnit::Binary,
                        KeyCode::Char('i') | KeyCode::Char('I') => {
                            self.selected_tab = Tab::Interface
                        }
                        KeyCode::Char('t') | KeyCode::Char('T') => self.selected_tab = Tab::Tcp,
                        _ => {}
                    },

                    Mode::SelectingInterface { filter, index } => match key.code {
                        KeyCode::Char(c) => {
                            filter.push(c);
                            *index = 0;
                        }
                        KeyCode::Backspace => {
                            filter.pop();
                        }
                        KeyCode::Up => {
                            if *index > 0 {
                                *index -= 1;
                            }
                        }
                        KeyCode::Down => {
                            let filtered_len = interface_name_vec
                                .iter()
                                .filter(|&name| name.contains(&*filter))
                                .count();
                            if *index + 1 < filtered_len {
                                *index += 1;
                            }
                        }

                        KeyCode::Enter => {
                            let name_match: Vec<_> = interface_name_vec
                                .iter()
                                .filter(|&name| name.contains(&*filter))
                                .collect();

                            if let Some(&selected_interface) = name_match.get(*index) {
                                self.selected_interface =
                                    InterfaceSelected::Interface(selected_interface.clone());
                                self.mode = Mode::Normal;
                            }
                        }

                        KeyCode::Esc => {
                            self.mode = Mode::Normal;
                            self.selected_interface = InterfaceSelected::All;
                        }
                        _ => {}
                    },
                }
                if self.enter_tick_active {
                    match key.code {
                        KeyCode::Char(c) => {
                            self.tick_value.push(c);
                        }
                        KeyCode::Backspace => {
                            self.tick_value.pop();
                        }
                        KeyCode::Enter => {
                            let value = self.tick_value.parse::<u64>();
                            if let Ok(v) = value {
                                self.tick_rate = Duration::from_millis(v);
                            }
                            self.enter_tick_active = false;
                        }
                        KeyCode::Esc => {
                            self.enter_tick_active = false;
                        }
                        _ => {}
                    }
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
        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .position(self.horizontal_scroll);
    }
    pub fn update_scroll_state(&mut self) {
        self.vertical_scroll_state = self.vertical_scroll_state.position(self.vertical_scroll);
        self.horizontal_scroll_state = self
            .horizontal_scroll_state
            .position(self.horizontal_scroll);
    }
    pub fn render(&mut self, frame: &mut Frame, data: &Vec<NetworkStats>) {
        match self.selected_tab {
            Tab::Interface => {
                crate::ui::draw_interface_mode(self, frame, data);
            }
            Tab::Tcp => {}
        }
    }
}
