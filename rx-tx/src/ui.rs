use crate::app;
use crate::app::*;
use crate::models::*;
use crate::parser::*;
use anyhow::{anyhow, Error, Ok, Result};
use clap::builder::Str;
use crossterm::event::{self, read, Event, KeyCode};
use ratatui::layout::Rect;
use ratatui::layout::{Alignment, Constraint, Layout, Margin};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols;
use ratatui::symbols::scrollbar;
use ratatui::text::{Line, Masked, Span};
use ratatui::widgets::block::title;
use ratatui::widgets::Axis;
use ratatui::widgets::Borders;
use ratatui::widgets::Chart;
use ratatui::widgets::Dataset;
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{text::Text, Frame};
use ratatui::{DefaultTerminal, Terminal};
use std::collections::HashMap;
use std::ops::Sub;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::vec;

pub fn draw_interface_mode(app: &mut App, frame: &mut Frame, data: &Vec<NetworkStats>) {
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

    let para = Paragraph::new(get_network_interfaces(data))
        .white()
        .scroll((app.vertical_scroll as u16, 0))
        .block(block("Interface"));

    let rx_rectangle = Paragraph::new(get_network_receive_data(app, data))
        .white()
        .scroll((app.vertical_scroll as u16, 0))
        .block(r_block("Received"));

    let tx_rectangle = Paragraph::new(get_network_transmit_data(app, data))
        .white()
        .scroll((app.vertical_scroll as u16, 0))
        .block(t_block("Transmit"));

    frame.render_widget(para, interface_rect);
    frame.render_widget(rx_rectangle, rx_rect);
    frame.render_widget(tx_rectangle, tx_rect);

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("|"))
            .end_symbol(Some("|")),
        rx_rect,
        &mut app.horizontal_scroll_state,
    );
}

pub fn render_graph(app: &mut App, frame: &mut Frame, area: Rect) {
    todo!()
}
