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
use std::time::{Duration, Instant};

use crate::parse_proc_net_dev;
use crate::parse_proc_net_tcp;
use crate::{get_network_interfaces, get_network_receive_data, get_network_transmit_data};
use crate::{NetworkStats, TcpStats};

#[derive(Default)]
pub struct App {
    pub current_tab: Tab,
    pub vertical_scroll_state: ScrollbarState,
    pub horizontal_scroll_state: ScrollbarState,
    pub horizontal_scroll: usize,
    pub vertical_scroll: usize,
}

#[derive(Default, Copy, Clone)]
pub enum Tab {
    #[default]
    Interface,
    Tcp,
}

impl Tab {
    fn titles() -> Vec<&'static str> {
        vec!["Interface Mode (I)", "TCP Mode (T)"]
    }
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let tick_rate = Duration::from_millis(250);
        let mut last_tick = Instant::now();

        loop {
            let vec_stats = parse_proc_net_dev()?;
            let _ = terminal.draw(|frame| self.render(frame, vec_stats));
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

    pub fn render(&mut self, frame: &mut Frame, line: Vec<NetworkStats>) {
        let area = frame.area();

        let block = |title: &'static str| Block::bordered().green().title(title.bold());
        let r_block = |title: &'static str| Block::bordered().blue().title(title.bold());
        let t_block = |title: &'static str| Block::bordered().red().title(title.bold());

        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(50),
            Constraint::Percentage(49),
        ])
        .split(area);

        let top_part = chunks[1];
        let second_part = chunks[2];
        let top_chunks = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(37),
            Constraint::Percentage(38),
        ])
        .split(top_part);
        let interface_rect = top_chunks[0];
        let rx_rect = top_chunks[1];
        let tx_rect = top_chunks[2];

        let title = Line::from("INTERFACE MODE (I)").centered().bold();
        frame.render_widget(title, chunks[0]);

        let para = Paragraph::new(get_network_interfaces(&line))
            .white()
            .scroll((self.vertical_scroll as u16, 0))
            .block(block("Interface"));

        let rx_rectangle = Paragraph::new(get_network_receive_data(&line))
            .white()
            .scroll((self.vertical_scroll as u16, 0))
            .block(r_block("Received"));

        let tx_rectangle = Paragraph::new(get_network_transmit_data(&line))
            .white()
            .scroll((self.vertical_scroll as u16, 0))
            .block(t_block("Transmit"));

        frame.render_widget(para, interface_rect);
        frame.render_widget(rx_rectangle, rx_rect);
        frame.render_widget(tx_rectangle, tx_rect);

        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓")),
            chunks[1],
            &mut self.vertical_scroll_state,
        );
    }
}
