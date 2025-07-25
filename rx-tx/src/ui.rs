use crate::app;
use crate::app::*;
use crate::models;
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
use ratatui::widgets::{
    Axis, BorderType, Borders, Chart, Dataset, HighlightSpacing, List, ListItem, ListState, Tabs,
    Widget,
};
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

    let it_block = |title: &'static str| {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green))
            .title(title.bold().into_centered_line())
    };
    let r_block = |title: String| {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .blue()
            .title(title.bold().into_centered_line())
    };
    let t_block = |title: String| {
        Block::bordered()
            .border_type(BorderType::Rounded)
            .red()
            .title(title.bold().into_centered_line())
    };

    let chunks = if !app.is_full_screen {
        Layout::vertical([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
            Constraint::Length(1),
        ])
        .split(area)
    } else {
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area)
    };

    let data_part = chunks[0];
    let graph_part = chunks[1];
    let status_bar_part = chunks[2];

    if !app.is_full_screen {
        let tab_titles = Tab::titles();
        let mut spans = Vec::new();

        for title in tab_titles.iter() {
            spans.push(Span::styled(
                format!(" {} ", title),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ));
            spans.push(Span::raw(" | "));
        }

        spans.push(Span::styled(
            "e: fullscreen",
            Style::default().fg(Color::DarkGray),
        ));
        spans.push(Span::raw(" | "));

        spans.push(Span::styled(
            "q: quit",
            Style::default().fg(Color::DarkGray),
        ));
        let paragraph = if app.enter_tick_active {
            Paragraph::new(Line::from(vec![
                Span::styled("> Tick Rate: ", Style::default().fg(Color::LightBlue)),
                Span::raw(app.tick_value.as_str()),
            ]))
            .alignment(Alignment::Left)
        } else {
            let tab_titles = Tab::titles();
            let mut spans = Vec::new();

            for title in tab_titles.iter() {
                spans.push(Span::styled(
                    format!(" {} ", title),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
                spans.push(Span::raw(" | "));
            }

            let tick_millis = app.tick_rate.as_millis();
            let tick_display = if tick_millis >= 1000 {
                format!("Tick(k): {:.1}s", (tick_millis as f64) / 1000.0)
            } else {
                format!("Tick(k): {}ms", tick_millis)
            };
            spans.push(Span::styled(
                tick_display,
                Style::default().fg(Color::LightBlue),
            ));
            spans.push(Span::raw(" | "));

            spans.push(Span::styled(
                "e: fullscreen",
                Style::default().fg(Color::DarkGray),
            ));
            spans.push(Span::raw(" | "));

            spans.push(Span::styled(
                "q: quit",
                Style::default().fg(Color::DarkGray),
            ));

            spans.push(Span::raw(" | "));
            Paragraph::new(Line::from(spans)).alignment(Alignment::Center)
        };

        frame.render_widget(paragraph, status_bar_part);
    }

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
        .block(r_block("Received".to_string()))
        .scroll((app.vertical_scroll as u16, app.horizontal_scroll as u16));

    let tx_para = Paragraph::new(get_network_transmit_data(app, data))
        .white()
        .block(t_block("Transmit".to_string()))
        .scroll((app.vertical_scroll as u16, app.horizontal_scroll as u16));

    let interface_names: Vec<String> = data
        .iter()
        .map(|interface| interface.name.clone())
        .collect();

    if let app::Mode::SelectingInterface { filter, index } = &app.mode {
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
            let sel = (*index).min(filtered.len() - 1);
            state.select(Some(sel));
        }
        let list = List::new(items)
            .block(it_block("Select Interface"))
            .highlight_symbol(">> ")
            .highlight_style(Style::default().fg(Color::Yellow))
            .highlight_spacing(HighlightSpacing::Always);
        frame.render_stateful_widget(list, interface_rect, &mut state);
    }

    if let app::Mode::Normal = &app.mode {
        match &app.selected_interface {
            app::InterfaceSelected::Interface(name) => {
                let i_name = app.interface_name.clone();

                //TODO: have to think other design, rather thaan whatever this is
                let it_para = Paragraph::new(i_name)
                    .white()
                    .block(it_block("Interface"))
                    .alignment(Alignment::Center);

                frame.render_widget(it_para, interface_rect);
            }
            app::InterfaceSelected::All => {
                let items: Vec<ListItem> = interface_names
                    .iter()
                    .map(|x| ListItem::new(format!(" {}", x)))
                    .collect();
                let list = List::new(items)
                    .block(it_block("Interface(f)"))
                    .style(Style::default().fg(Color::White));
                frame.render_widget(list, interface_rect);
            }
        }
    }
    let interface_name = app.interface_name.clone();

    match &app.selected_interface {
        app::InterfaceSelected::Interface(it) => {
            let rx_para = Paragraph::new(get_selected_network_receive_data(app, data))
                .white()
                .block(r_block(interface_name.clone()))
                .scroll((app.vertical_scroll as u16, app.horizontal_scroll as u16));

            frame.render_widget(rx_para, rx_rect);

            let tx_para = Paragraph::new(get_selected_network_transmit_data(app, data))
                .white()
                .block(t_block(interface_name))
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
