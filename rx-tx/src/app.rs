use crate::app;
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
use std::collections::HashMap;
use std::result::Result::Ok;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::vec;

#[derive(Clone, Debug)]
pub struct App {
    pub main_tab_focus: bool,
    pub interface_speeds: HashMap<String, (f64, f64)>,
    pub edit_rx_mode: bool,
    pub edit_tx_mode: bool,
    pub speed_input: String,
    pub speed_input_field: SpeedInputField,
    pub editing_interface: Option<String>,
    pub hostname_cache_arc: Arc<Mutex<HashMap<[u8; 4], String>>>,
    pub show_help: bool,
    pub enter_tick_active: bool,
    pub tick_rate: Duration,
    pub tick_value: String,
    pub mode: Mode,
    pub selection_state: ListState,
    pub selected_interface: InterfaceSelected,
    pub prev_stats: Option<Vec<NetworkStats>>,
    pub tcp_stats: Option<Vec<TcpStats>>,
    pub rx_data: HashMap<String, Vec<(f64, f64)>>,
    pub tx_data: HashMap<String, Vec<(f64, f64)>>,
    pub start_time: Instant,
    pub last_sample_time: f64,
    pub window: [f64; 2],
    pub raw_bytes: bool,
    pub byte_unit: ByteUnit,
    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    pub horizontal_scroll: usize,
    pub vertical_scroll: usize,
    pub tcp_vertical_scroll_state: ScrollbarState,
    pub tcp_vertical_scroll: usize,
    pub focus: Focus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpeedInputField {
    RX,
    TX,
}

impl Default for App {
    fn default() -> Self {
        Self {
            speed_input_field: SpeedInputField::RX,
            main_tab_focus: true,
            interface_speeds: get_interface_speed(),
            edit_tx_mode: false,
            edit_rx_mode: false,
            speed_input: String::new(),
            editing_interface: None,
            hostname_cache_arc: Arc::new(Mutex::new(HashMap::new())),
            focus: Focus::Interfaces,
            show_help: false,
            tick_value: String::new(),
            enter_tick_active: false,
            tick_rate: Duration::from_millis(1800),
            selection_state: {
                let mut state = ListState::default();
                state.select(Some(0));
                state
            },
            mode: Mode::Normal,
            selected_interface: InterfaceSelected::All,
            prev_stats: None,
            tcp_stats: None,
            rx_data: HashMap::new(),
            tx_data: HashMap::new(),
            start_time: Instant::now(),
            last_sample_time: 0.0,
            window: [0.0, 60.0],
            raw_bytes: false,
            byte_unit: ByteUnit::default(),
            vertical_scroll_state: ScrollbarState::new(0),
            horizontal_scroll_state: ScrollbarState::new(0),
            horizontal_scroll: 0,
            vertical_scroll: 0,
            tcp_vertical_scroll_state: ScrollbarState::new(0),
            tcp_vertical_scroll: 0,
        }
    }
}

#[derive(Default, Clone, Debug)]
pub enum ByteUnit {
    #[default]
    Binary,
    Decimal,
}

#[derive(Clone, Debug)]
pub enum Mode {
    Normal,
    SelectingInterface { filter: String, index: usize },
}

#[derive(Clone, Debug)]
pub enum InterfaceSelected {
    All,
    Interface(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Focus {
    Interfaces,
    TcpTable,
}

impl App {
    pub fn get_stuff(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let now = self.start_time.elapsed().as_secs_f64();
        self.window = [now - 5.0, now];

        let net_vec_stats = parse_proc_net_dev()?;
        let tcp_stats = parse_proc_net_tcp()?;
        self.tcp_stats = Some(tcp_stats);

        if let Some(prev_data) = &self.prev_stats {
            for (prev, new) in prev_data.iter().zip(net_vec_stats.iter()) {
                let rx_delta = new.receive.bytes.saturating_sub(prev.receive.bytes);
                let tx_delta = new.transmit.bytes.saturating_sub(prev.transmit.bytes);

                self.rx_data
                    .entry(new.name.clone())
                    .or_insert_with(Vec::new)
                    .push((now, rx_delta as f64));

                self.tx_data
                    .entry(new.name.clone())
                    .or_insert_with(Vec::new)
                    .push((now, tx_delta as f64));

                if let Some(rx_vec) = self.rx_data.get_mut(&new.name) {
                    if rx_vec.len() > 100 {
                        rx_vec.remove(0);
                    }
                }
                if let Some(tx_vec) = self.tx_data.get_mut(&new.name) {
                    if tx_vec.len() > 100 {
                        tx_vec.remove(0);
                    }
                }
            }
        }
        self.prev_stats = Some(net_vec_stats);

        Ok(())
    }

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
            let tick_rate = self.tick_rate;
            let latest_stats = self.prev_stats.clone().unwrap();
            if let Some(tcp_stats) = self.tcp_stats.clone() {
                let _ = terminal.draw(|frame| self.render(frame, &latest_stats, &tcp_stats));
            }

            let mut timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if timeout == Duration::ZERO {
                timeout = Duration::from_millis(5);
            }

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match &mut self.mode {
                        Mode::Normal => match key.code {
                            KeyCode::Char('R') => {
                                if let InterfaceSelected::Interface(ref name) =
                                    self.selected_interface
                                {
                                    self.edit_rx_mode = true;
                                    self.editing_interface = Some(name.clone());

                                    if let Some((rx, _)) = self.interface_speeds.get(name) {
                                        self.speed_input = rx.to_string();
                                    } else {
                                        self.speed_input.clear();
                                    }
                                }
                            }
                            KeyCode::Char('T') => {
                                if let InterfaceSelected::Interface(ref name) =
                                    self.selected_interface
                                {
                                    self.edit_tx_mode = true;
                                    self.editing_interface = Some(name.clone());

                                    if let Some((_, tx)) = self.interface_speeds.get(name) {
                                        self.speed_input = tx.to_string();
                                    } else {
                                        self.speed_input.clear();
                                    }
                                }
                            }

                            KeyCode::Tab => {
                                if self.main_tab_focus {
                                    self.focus = match self.focus {
                                        Focus::Interfaces => Focus::TcpTable,
                                        Focus::TcpTable => Focus::Interfaces,
                                    };
                                }
                            }
                            KeyCode::Char('f') => {
                                self.mode = Mode::SelectingInterface {
                                    filter: String::new(),
                                    index: 0,
                                }
                            }
                            KeyCode::Char('K') => {
                                self.enter_tick_active = true;
                                self.tick_value.clear();
                                continue;
                            }
                            KeyCode::Char('q') => {
                                break;
                            }
                            KeyCode::Left => self.scroll_left(),
                            KeyCode::Down => match self.focus {
                                Focus::Interfaces => self.scroll_down(),
                                Focus::TcpTable => {
                                    self.tcp_tablescroll_down();
                                }
                            },
                            KeyCode::Up => match self.focus {
                                Focus::Interfaces => self.scroll_up(),
                                Focus::TcpTable => self.tcp_tablescroll_up(),
                            },
                            KeyCode::Char('j') => match self.focus {
                                Focus::Interfaces => self.scroll_down(),
                                Focus::TcpTable => {
                                    self.tcp_tablescroll_down();
                                }
                            },
                            KeyCode::Char('k') => match self.focus {
                                Focus::Interfaces => self.scroll_up(),
                                Focus::TcpTable => self.tcp_tablescroll_up(),
                            },

                            KeyCode::Right => self.scroll_right(),
                            KeyCode::Char('r') => self.raw_bytes = !self.raw_bytes,
                            KeyCode::Char('d') => self.byte_unit = ByteUnit::Decimal,
                            KeyCode::Char('b') => self.byte_unit = ByteUnit::Binary,
                            KeyCode::Char('?') | KeyCode::Char('h') => {
                                self.show_help = !self.show_help
                            }
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
                                    self.get_stuff(terminal)?;
                                }
                            }
                            KeyCode::Esc => {
                                self.mode = Mode::Normal;
                                self.selected_interface = InterfaceSelected::All;
                            }
                            _ => {}
                        },
                    }
                    if self.show_help {
                        match key.code {
                            KeyCode::Esc => {
                                self.show_help = false;
                            }
                            _ => {}
                        }
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
                                if let Ok(v) = self.tick_value.parse::<u64>() {
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
                    if self.edit_rx_mode {
                        match key.code {
                            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                                self.speed_input.push(c);
                            }
                            KeyCode::Backspace => {
                                self.speed_input.pop();
                            }
                            KeyCode::Enter => {
                                if let Some(ref interface) = self.editing_interface {
                                    if let Ok(rx) = self.speed_input.parse::<f64>() {
                                        if rx > 0.0 {
                                            let tx = self
                                                .interface_speeds
                                                .get(interface)
                                                .map(|(_, tx)| *tx)
                                                .unwrap_or(rx);

                                            self.interface_speeds
                                                .insert(interface.clone(), (rx, tx));
                                            let _ = save_interface_speeds(&self.interface_speeds);
                                        }
                                    }
                                }
                                self.edit_rx_mode = false;
                                self.editing_interface = None;
                            }
                            KeyCode::Esc => {
                                self.edit_rx_mode = false;
                                self.editing_interface = None;
                            }
                            _ => {}
                        }
                    }

                    if self.edit_tx_mode {
                        match key.code {
                            KeyCode::Char(c) if c.is_ascii_digit() || c == '.' => {
                                self.speed_input.push(c);
                            }
                            KeyCode::Backspace => {
                                self.speed_input.pop();
                            }
                            KeyCode::Enter => {
                                if let Some(ref interface) = self.editing_interface {
                                    if let Ok(tx) = self.speed_input.parse::<f64>() {
                                        if tx > 0.0 {
                                            let rx = self
                                                .interface_speeds
                                                .get(interface)
                                                .map(|(rx, _)| *rx)
                                                .unwrap_or(tx);

                                            self.interface_speeds
                                                .insert(interface.clone(), (rx, tx));
                                            let _ = save_interface_speeds(&self.interface_speeds);
                                        }
                                    }
                                }
                                self.edit_tx_mode = false;
                                self.editing_interface = None;
                            }
                            KeyCode::Esc => {
                                self.edit_tx_mode = false;
                                self.editing_interface = None;
                            }
                            _ => {}
                        }
                    }
                }
            }

            if last_tick.elapsed() >= self.tick_rate {
                self.get_stuff(terminal)?;
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    pub fn get_rx_limit(&self, interface: &str) -> f64 {
        self.interface_speeds
            .get(interface)
            .map(|(rx, _)| *rx / 8.0)
            .unwrap_or(125.0) // Default is 125Mbps btw
    }

    pub fn get_tx_limit(&self, interface: &str) -> f64 {
        self.interface_speeds
            .get(interface)
            .map(|(_, tx)| *tx / 8.0)
            .unwrap_or(125.0) // Default is 125Mbps btw
    }

    pub fn scroll_up(&mut self) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(1);
        self.update_scroll_state();
    }

    pub fn tcp_tablescroll_up(&mut self) {
        self.tcp_vertical_scroll = self.tcp_vertical_scroll.saturating_sub(1);
        self.tcp_update_scroll_state();
    }

    pub fn tcp_tablescroll_down(&mut self) {
        self.tcp_vertical_scroll = self.tcp_vertical_scroll.saturating_add(1);
        self.tcp_update_scroll_state();
    }

    pub fn tcp_update_scroll_state(&mut self) {
        self.tcp_vertical_scroll_state = self
            .tcp_vertical_scroll_state
            .position(self.tcp_vertical_scroll);
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
    pub fn render(
        &mut self,
        frame: &mut Frame,
        net_data: &Vec<NetworkStats>,
        tcp_data: &Vec<TcpStats>,
    ) {
        crate::ui::draw_interface_mode(self, frame, net_data, tcp_data);
    }
}
