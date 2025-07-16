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
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Chart;
use ratatui::widgets::Dataset;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::Tabs;
use ratatui::widgets::Widget;
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

    let block = |title: &'static str| {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .green()
            .title(title.bold())
    };
    let r_block = |title: &'static str| {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .blue()
            .title(title.bold())
    };
    let t_block = |title: &'static str| {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .red()
            .title(title.bold())
    };

    let chunks =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    let top_part = chunks[0];

    let second_part = chunks[1];
    let split_graph = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(second_part);
    let top_chunks = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(37),
        Constraint::Percentage(38),
    ])
    .split(top_part);
    let interface_rect = top_chunks[0];
    let rx_rect = top_chunks[1];
    let tx_rect = top_chunks[2];

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
    render_rx_graph(app, frame, split_graph[0]);
    render_tx_graph(app, frame, split_graph[1]);
}

pub fn render_rx_graph(app: &mut App, frame: &mut Frame, area: Rect) {
    let rx_max = app.rx_data.iter().map(|&(_, y)| y).fold(0.0, f64::max);

    let datasets = vec![Dataset::default()
        .name("RX")
        .marker(symbols::Marker::Block)
        .style(Style::default().fg(Color::Blue))
        .graph_type(ratatui::widgets::GraphType::Bar)
        .data(&app.rx_data)];

    let chart = Chart::new(datasets)
        .block(Block::bordered().border_type(BorderType::Rounded))
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(app.window),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, rx_max]),
        );

    frame.render_widget(chart, area);
}
pub fn render_tx_graph(app: &mut App, frame: &mut Frame, area: Rect) {
    let tx_max = app.tx_data.iter().map(|&(_, y)| y).fold(0.0, f64::max);
    let datasets = vec![Dataset::default()
        .name("TX")
        .marker(symbols::Marker::Block)
        .graph_type(ratatui::widgets::GraphType::Bar)
        .style(Style::default().fg(Color::Red))
        .data(&app.tx_data)];

    let chart = Chart::new(datasets)
        .block(Block::bordered().border_type(BorderType::Rounded))
        .x_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds(app.window),
        )
        .y_axis(
            Axis::default()
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, tx_max]),
        );

    frame.render_widget(chart, area);
}
