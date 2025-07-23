use crate::app;
use crate::app::*;
use crate::models::*;
use crate::parser::*;
use anyhow::{anyhow, Error, Ok, Result};
use clap::builder::Str;
use crossterm::event::{self, read, Event, KeyCode};
use ratatui::buffer;
use ratatui::buffer::Buffer;
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
use ratatui::widgets::ListState;
use ratatui::widgets::Tabs;
use ratatui::widgets::Widget;
use ratatui::widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use ratatui::{text::Text, Frame};
use ratatui::{DefaultTerminal, Terminal};
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::format;
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

    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Percentage(50),
        Constraint::Percentage(50),
    ])
    .split(area);

    let tab_part = chunks[0];
    let data_part = chunks[1];
    let graph_part = chunks[2];

    let split_graph = Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(graph_part);
    let top_chunks = Layout::horizontal([
        Constraint::Percentage(20),
        Constraint::Length(1),
        Constraint::Percentage(39),
        Constraint::Length(1),
        Constraint::Percentage(39),
    ])
    .split(data_part);

    let interface_rect = top_chunks[0];
    let rx_rect = top_chunks[2];
    let tx_rect = top_chunks[4];

    let rx_para = Paragraph::new(get_network_receive_data(app, data))
        .white()
        .block(r_block("Received"))
        .scroll((app.vertical_scroll as u16, app.horizontal_scroll as u16));

    let tx_para = Paragraph::new(get_network_transmit_data(app, data))
        .white()
        .block(t_block("Transmit"))
        .scroll((app.vertical_scroll as u16, app.horizontal_scroll as u16));

    let titles: Vec<_> = Tab::titles().iter().map(|&s| s).collect();
    let tab =
        Tabs::new(titles).block(Block::default().style(Style::default().fg(Color::LightYellow)));

    frame.render_widget(tab, tab_part);

    let interface_names: Vec<String> = data
        .iter()
        .map(|interface| interface.name.clone())
        .collect();

    let (list_items, block_title, mut state) = match &app.mode {
        app::Mode::SelectingInterface { filter, index } => {
            let filtered: Vec<_> = interface_names
                .iter()
                .filter(|s| s.contains(filter))
                .collect();

            let items: Vec<ListItem> = filtered
                .iter()
                .map(|x| ListItem::new(format!(" {}", x)))
                .collect();

            let mut state = ListState::default();
            if !filtered.is_empty() {
                let s = (*index).min(filtered.len().saturating_sub(1));
                state.select(Some(s));
            }
            (items, "Select Interface", state)
        }
        app::Mode::Normal => {
            let state = ListState::default();
            let items: Vec<ListItem> = interface_names
                .iter()
                .map(|x| ListItem::new(format!(" {}", x)))
                .collect();

            (items, "Interface", state)
        }
    };

    let mut list = List::new(list_items)
        .block(block(block_title))
        .style(Style::default().fg(Color::White));
    match &app.mode {
        app::Mode::SelectingInterface { .. } => {
            list = list
                .highlight_symbol(">>")
                .highlight_style(Style::default().fg(Color::White))
                .highlight_spacing(ratatui::widgets::HighlightSpacing::Always);

            frame.render_stateful_widget(list, interface_rect, &mut state);
        }
        app::Mode::Normal => {
            frame.render_widget(list, interface_rect);
        }
    }

    match &app.selected_interface {
        app::InterfaceSelected::Interface(it) => {
            let rx_para = Paragraph::new(get_selected_network_receive_data(app, data))
                .white()
                .block(r_block("Received"))
                .scroll((app.vertical_scroll as u16, app.horizontal_scroll as u16));

            frame.render_widget(rx_para, rx_rect);

            let tx_para = Paragraph::new(get_selected_network_transmit_data(app, data))
                .white()
                .block(t_block("Transmit"))
                .scroll((app.vertical_scroll as u16, app.horizontal_scroll as u16));

            frame.render_widget(tx_para, tx_rect);
        }
        app::InterfaceSelected::All => {
            frame.render_widget(rx_para, rx_rect);
            frame.render_widget(tx_para, tx_rect);
        }
    }

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("▌")
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"))
            .track_symbol(Some("·")),
        interface_rect.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut app.vertical_scroll_state,
    );

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .thumb_symbol("─")
            .begin_symbol(Some("«"))
            .end_symbol(Some("»"))
            .track_symbol(Some("·")),
        interface_rect.inner(Margin {
            vertical: 0,
            horizontal: 1,
        }),
        &mut app.horizontal_scroll_state,
    );

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("▌")
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"))
            .track_symbol(Some("·")),
        rx_rect.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut app.vertical_scroll_state,
    );

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .thumb_symbol("─")
            .begin_symbol(Some("«"))
            .end_symbol(Some("»"))
            .track_symbol(Some("·")),
        rx_rect.inner(Margin {
            vertical: 0,
            horizontal: 1,
        }),
        &mut app.horizontal_scroll_state,
    );

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("▌")
            .begin_symbol(Some("^"))
            .end_symbol(Some("v"))
            .track_symbol(Some("·")),
        tx_rect.inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut app.vertical_scroll_state,
    );

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::HorizontalBottom)
            .thumb_symbol("─")
            .begin_symbol(Some("«"))
            .end_symbol(Some("»"))
            .track_symbol(Some("·")),
        tx_rect.inner(Margin {
            vertical: 0,
            horizontal: 1,
        }),
        &mut app.horizontal_scroll_state,
    );

    draw_rx_graph(app, frame, split_graph[0]);
    draw_tx_graph(app, frame, split_graph[1]);
}

pub fn draw_rx_graph(app: &mut App, frame: &mut Frame, area: Rect) {
    let rx_max = app.rx_data.iter().map(|&(_, y)| y).fold(0.0, f64::max);

    let datasets = vec![Dataset::default()
        .name(app.interface_name.clone())
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
pub fn draw_tx_graph(app: &mut App, frame: &mut Frame, area: Rect) {
    let tx_max = app.tx_data.iter().map(|&(_, y)| y).fold(0.0, f64::max);
    let datasets = vec![Dataset::default()
        .name(app.interface_name.clone())
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
