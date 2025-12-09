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
use ratatui::widgets::Cell;
use ratatui::widgets::Clear;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::Wrap;
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
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::ops::Sub;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::vec;

fn draw_tick_mode(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let popup_area = centered_rect(40, 30, area);

    frame.render_widget(Clear, popup_area);

    let tick_millis = app.tick_rate.as_millis();
    let current_tick = if tick_millis >= 1000 {
        format!("{:.1}s", (tick_millis as f64) / 1000.0)
    } else {
        format!("{}ms", tick_millis)
    };

    let tick_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("  Current: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                current_tick,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  New Rate: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                if app.tick_value.is_empty() {
                    "_".to_string()
                } else {
                    format!("{}█", app.tick_value)
                },
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" ms", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                "  Enter ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("to apply  ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Esc ",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::styled("to cancel", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let tick_popup = Paragraph::new(tick_text)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" ⏱ SET TICK RATE ")
                .title_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ),
        )
        .alignment(Alignment::Left);

    frame.render_widget(tick_popup, popup_area);
}

pub fn draw_interface_mode(
    app: &mut App,
    frame: &mut Frame,
    data: &Vec<NetworkStats>,
    tcp_data: &Vec<TcpStats>,
) {
    let mut rx_peak_speed: HashMap<String, f64> = HashMap::new();
    let mut tx_peak_speed: HashMap<String, f64> = HashMap::new();
    let mut rx_avg_speed: HashMap<String, f64> = HashMap::new();
    let mut tx_avg_speed: HashMap<String, f64> = HashMap::new();

    let interface_border_color = if app.focus == Focus::Interfaces {
        Color::Rgb(100, 200, 255)
    } else {
        Color::Rgb(80, 80, 80)
    };

    let tcp_border_color = if app.focus == Focus::TcpTable {
        Color::Rgb(100, 200, 255)
    } else {
        Color::Rgb(80, 80, 80)
    };

    let byte_unit = app.byte_unit.clone();
    let area = frame.area();
    let uptime = parse_uptime().unwrap_or(String::new());
    let tick_millis = app.tick_rate.as_millis();
    let tick_display = if tick_millis >= 1000 {
        format!("{:.1}s", (tick_millis as f64) / 1000.0)
    } else {
        format!("{}ms", tick_millis)
    };

    let chunks =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);
    let main_part = chunks[0];
    let tcp_area = chunks[1];

    let main_split =
        Layout::horizontal([Constraint::Length(25), Constraint::Fill(1)]).split(main_part);

    let list_area = main_split[0];
    let detail_area = main_split[1];

    let rx_data_strings = get_network_receive_data(app, data);
    let tx_data_lines = get_network_transmit_data(app, data);

    let interface_names: Vec<String> = data.iter().map(|i| i.name.clone()).collect();

    match &app.mode {
        Mode::SelectingInterface { filter, index } => {
            let filtered: Vec<_> = interface_names
                .iter()
                .enumerate()
                .filter(|(_, name)| name.contains(filter))
                .collect();

            let items: Vec<ListItem> = filtered
                .iter()
                .enumerate()
                .map(|(display_idx, (original_idx, name))| {
                    let rx_speed_str = extract_speed_from_line(
                        &rx_data_strings
                            .get(*original_idx)
                            .cloned()
                            .unwrap_or(Line::from("0 B/s")),
                    );
                    let tx_speed_str = extract_speed_from_line(
                        &tx_data_lines
                            .get(*original_idx)
                            .cloned()
                            .unwrap_or(Line::from("0 B/s")),
                    );

                    let rx_load = parse_speed(&rx_speed_str, Some(5.0));
                    let tx_load = parse_speed(&tx_speed_str, Some(5.0));
                    let is_active = rx_load > 0.01 || tx_load > 0.01;

                    let activity = if is_active {
                        Span::styled(
                            "⚡",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw(" ")
                    };

                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(
                                format!("{:>2}.", display_idx + 1),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::raw(" "),
                            Span::styled(format!("{:<16}", name), Style::default().fg(Color::Cyan)),
                            activity,
                        ]),
                        Line::from(""),
                    ])
                })
                .collect();

            let mut state = ListState::default();
            if !filtered.is_empty() {
                let sel = (*index).min(filtered.len() - 1);
                state.select(Some(sel));
            }

            let list = List::new(items)
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(format!(" 🔍 Filter: {} ", filter))
                        .title_style(
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                )
                .highlight_symbol("➣ ")
                .highlight_style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_spacing(HighlightSpacing::Always);

            frame.render_stateful_widget(list, list_area, &mut state);

            let mut scrollbar_state = ScrollbarState::new(filtered.len()).position(*index);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(Some("┊"))
                    .thumb_symbol("┃"),
                list_area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
        Mode::Normal => {
            let items: Vec<ListItem> = interface_names
                .iter()
                .enumerate()
                .map(|(idx, name)| {
                    let rx_speed_str = extract_speed_from_line(
                        &rx_data_strings
                            .get(idx)
                            .cloned()
                            .unwrap_or(Line::from("0 B/s")),
                    );
                    let tx_speed_str = extract_speed_from_line(
                        &tx_data_lines
                            .get(idx)
                            .cloned()
                            .unwrap_or(Line::from("0 B/s")),
                    );

                    let rx_load = parse_speed(&rx_speed_str, Some(5.0));
                    let tx_load = parse_speed(&tx_speed_str, Some(5.0));
                    let is_active = rx_load > 0.01 || tx_load > 0.01;

                    let activity = if is_active {
                        Span::styled(
                            "⚡",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw(" ")
                    };

                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(
                                format!("{:>2}.", idx + 1),
                                Style::default().fg(Color::DarkGray),
                            ),
                            Span::raw(" "),
                            Span::styled(format!("{:<16}", name), Style::default().fg(Color::Cyan)),
                            activity,
                        ]),
                        Line::from(""),
                    ])
                })
                .collect();

            let mut state = ListState::default();
            state.select(Some(app.vertical_scroll));

            let list = List::new(items).block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(format!(" 📡 INTERFACES (f)"))
                    .title_style(
                        Style::default()
                            .fg(interface_border_color)
                            .add_modifier(Modifier::BOLD),
                    ),
            );

            frame.render_stateful_widget(list, list_area, &mut state);
            let scroll_bar_state = frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(None)
                    .thumb_symbol("┃"),
                list_area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut app.vertical_scroll_state,
            );
        }
    }

    match app.selected_interface.clone() {
        InterfaceSelected::Interface(selected_name) => {
            if let Some(interface_data) = data.iter().find(|i| i.name == selected_name) {
                let idx = data.iter().position(|i| i.name == selected_name).unwrap();

                let rx_speed_str = extract_speed_from_line(
                    &rx_data_strings
                        .get(idx)
                        .cloned()
                        .unwrap_or(Line::from("0 B/s")),
                );
                let tx_speed_str = extract_speed_from_line(
                    &tx_data_lines
                        .get(idx)
                        .cloned()
                        .unwrap_or(Line::from("0 B/s")),
                );

                let bar_width = (detail_area.width as usize).saturating_sub(30);
                let rx_load = parse_speed(&rx_speed_str, Some(5.0));
                let tx_load = parse_speed(&tx_speed_str, Some(5.0));

                let detail_chunks = Layout::vertical([
                    Constraint::Length(5),
                    Constraint::Length(5),
                    Constraint::Fill(1),
                ])
                .split(detail_area);

                let rx_current = parse_speed(&rx_speed_str, None);
                let tx_current = parse_speed(&tx_speed_str, None);

                let rx_peak = rx_peak_speed.entry(selected_name.clone()).or_insert(0.0);
                if rx_current > *rx_peak {
                    *rx_peak = rx_current;
                }
                let tx_peak = tx_peak_speed.entry(selected_name.clone()).or_insert(0.0);
                if tx_current > *tx_peak {
                    *tx_peak = tx_current;
                }

                let rx_avg = rx_avg_speed.entry(selected_name.clone()).or_insert(0.0);
                let tx_avg = tx_avg_speed.entry(selected_name.clone()).or_insert(0.0);

                *rx_avg = (*rx_avg * 0.95) + (rx_current * 0.05);
                *tx_avg = (*tx_avg * 0.95) + (tx_current * 0.05);

                let rx_peak_str = format_speed_mbps(*rx_peak);
                let rx_avg_str = format_speed_mbps(*rx_avg);
                let tx_peak_str = format_speed_mbps(*tx_peak);
                let tx_avg_str = format_speed_mbps(*tx_avg);

                let rx_para = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            make_bar(rx_load, bar_width),
                            Style::default().fg(Color::Green),
                        ),
                        Span::raw(" ▼"),
                    ]),
                    Line::from(""),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(vec![
                            Span::styled(
                                " 📥 RX ",
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Cur: ", Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                format!("{:<10}", rx_speed_str),
                                Style::default()
                                    .fg(Color::Green)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Peak: ", Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                format!("{:<10}", rx_peak_str),
                                Style::default().fg(Color::Yellow),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Avg: ", Style::default().fg(Color::DarkGray)),
                            Span::styled(rx_avg_str, Style::default().fg(Color::Cyan)),
                            Span::raw(" "),
                        ])
                        .title_top(
                            Line::from(vec![
                                Span::styled(" Tick: ", Style::default().fg(Color::DarkGray)),
                                Span::styled(
                                    tick_display,
                                    Style::default()
                                        .fg(Color::Cyan)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(" (K) ", Style::default().fg(Color::DarkGray)),
                            ])
                            .right_aligned(),
                        ),
                )
                .alignment(Alignment::Left);

                frame.render_widget(rx_para, detail_chunks[0]);
                let tx_para = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            make_bar(tx_load, bar_width),
                            Style::default().fg(Color::Blue),
                        ),
                        Span::raw(" ▲"),
                    ]),
                    Line::from(""),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(vec![
                            Span::styled(
                                " 📤 TX ",
                                Style::default()
                                    .fg(Color::Blue)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Cur: ", Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                format!("{:<10}", tx_speed_str),
                                Style::default()
                                    .fg(Color::Blue)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Peak: ", Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                format!("{:<10}", tx_peak_str),
                                Style::default().fg(Color::Yellow),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Avg: ", Style::default().fg(Color::DarkGray)),
                            Span::styled(tx_avg_str, Style::default().fg(Color::Cyan)),
                            Span::raw(" "),
                        ])
                        .blue(),
                );
                frame.render_widget(tx_para, detail_chunks[1]);

                let total = interface_data.receive.bytes + interface_data.transmit.bytes;
                let total_str = interface_data.receive.display(app, Some(total));
                let rx_bytes_str = interface_data.receive.display(app, None);
                let tx_bytes_str = interface_data.transmit.display(app, None);

                let stats_columns = Layout::horizontal([
                    Constraint::Percentage(33),
                    Constraint::Percentage(33),
                    Constraint::Percentage(34),
                ])
                .split(detail_chunks[2]);

                let left_col = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled("  Name        : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            &interface_data.name,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Total       : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(total_str, Style::default().fg(Color::Yellow)),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  RX Bytes    : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(rx_bytes_str, Style::default().fg(Color::Green)),
                    ]),
                    Line::from(vec![
                        Span::styled("  RX Packets  : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.receive.packets),
                            Style::default().fg(Color::Green),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  TX Bytes    : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(tx_bytes_str, Style::default().fg(Color::Blue)),
                    ]),
                    Line::from(vec![
                        Span::styled("  TX Packets  : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.transmit.packets),
                            Style::default().fg(Color::Blue),
                        ),
                    ]),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(" 📊 INFO ")
                        .cyan(),
                );

                let middle_col = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled("  Errors      : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.receive.errs),
                            if interface_data.receive.errs > 0 {
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            },
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Drops       : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.receive.drop),
                            if interface_data.receive.drop > 0 {
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            },
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  FIFO        : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.receive.fifo),
                            if interface_data.receive.fifo > 0 {
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            },
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Frame       : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.receive.frame),
                            if interface_data.receive.frame > 0 {
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            },
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  Compressed  : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.receive.compressed),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Multicast   : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.receive.multicast),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(" 📥 RX ")
                        .green(),
                );

                let right_col = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled("  Errors       : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.transmit.errs),
                            if interface_data.transmit.errs > 0 {
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            },
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Drops        : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.transmit.drop),
                            if interface_data.transmit.drop > 0 {
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            },
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  FIFO         : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.transmit.fifo),
                            if interface_data.transmit.fifo > 0 {
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            },
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Collisions   : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.transmit.colls),
                            if interface_data.transmit.colls > 0 {
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            },
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  Carrier      : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.transmit.carrier),
                            if interface_data.transmit.carrier > 0 {
                                Style::default()
                                    .fg(Color::Yellow)
                                    .add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            },
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Compressed   : ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.transmit.compressed),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(" 📤 TX ")
                        .blue(),
                );

                frame.render_widget(left_col, stats_columns[0]);
                frame.render_widget(middle_col, stats_columns[1]);
                frame.render_widget(right_col, stats_columns[2]);
            }
        }
        InterfaceSelected::All => {
            let total_rx: u64 = data.iter().map(|i| i.receive.bytes).sum();
            let total_tx: u64 = data.iter().map(|i| i.transmit.bytes).sum();
            let total_packets: u64 = data.iter().map(|i| i.receive.packets).sum();
            let summary_tx_val = if app.raw_bytes {
                total_tx.to_string()
            } else {
                format_bytes(total_tx, &byte_unit)
            };
            let summary_rx_val = if app.raw_bytes {
                total_rx.to_string()
            } else {
                format_bytes(total_rx, &byte_unit)
            };

            let summary = Paragraph::new(vec![
                Line::from(Span::styled(
                    "  ALL INTERFACES",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(vec![
                    Span::styled("  System Uptime : ", Style::default().fg(Color::DarkGray)),
                    Span::styled(uptime, Style::default().fg(Color::Cyan)),
                ]),
                Line::from(vec![
                    Span::styled("  Total RX      : ", Style::default().fg(Color::DarkGray)),
                    Span::styled(summary_rx_val, Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("  Total TX      : ", Style::default().fg(Color::DarkGray)),
                    Span::styled(summary_tx_val, Style::default().fg(Color::Blue)),
                ]),
                Line::from(vec![
                    Span::styled("  Total Packets : ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}", total_packets),
                        Style::default().fg(Color::White),
                    ),
                ]),
            ])
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title_top(Line::from(" 📊 OVERVIEW ").left_aligned())
                    .title_top(
                        Line::from(vec![
                            Span::styled(" Tick: ", Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                tick_display,
                                Style::default()
                                    .fg(Color::Cyan)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(" (K) ", Style::default().fg(Color::DarkGray)),
                        ])
                        .right_aligned(),
                    ),
            )
            .alignment(Alignment::Left);

            frame.render_widget(summary, detail_area);
        }
    }

    let mut state_counts: std::collections::BTreeMap<&str, usize> =
        std::collections::BTreeMap::new();

    for conn in tcp_data.iter() {
        let state = tcp_state_name(conn.state);
        *state_counts.entry(state).or_insert(0) += 1;

        let ip = conn.remote_ip;

        if ip == [0, 0, 0, 0] || ip == [127, 0, 0, 1] {
            continue;
        }

        let needs_lookup = {
            let cache = app.hostname_cache_arc.lock().unwrap();
            !cache.contains_key(&ip)
        };

        if needs_lookup {
            app.hostname_cache_arc
                .lock()
                .unwrap()
                .insert(ip, "resolving...".to_string());

            let cache = Arc::clone(&app.hostname_cache_arc);
            std::thread::spawn(move || {
                let hostname = resolve_hostname(&ip);
                cache.lock().unwrap().insert(ip, hostname);
            });
        }
    }

    let tcp_split =
        Layout::horizontal([Constraint::Length(25), Constraint::Fill(1)]).split(tcp_area);

    let mut summary_lines = vec![
        Line::from("  Connections").style(Style::default().fg(Color::Yellow)),
        Line::from(vec![
            Span::styled("  Total       : ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("{}", tcp_data.len()),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];

    for (state, count) in state_counts.iter() {
        let color = match *state {
            "ESTABLISHED" => Color::Green,
            "LISTEN" => Color::Cyan,
            "TIME_WAIT" => Color::Yellow,
            _ => Color::White,
        };
        summary_lines.push(Line::from(vec![
            Span::styled(
                format!("  {:<12}: ", state),
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled(format!("{}", count), Style::default().fg(color)),
        ]));
    }

    let summary = Paragraph::new(summary_lines).block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .title(" 📊 INFO ")
            .title_style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    frame.render_widget(summary, tcp_split[0]);

    let tcp_rows: Vec<Row> = tcp_data
        .iter()
        .map(|conn| {
            let local_addr = format!("{}:{}", format_ip(&conn.local_ip), conn.local_port);
            let remote_addr = format!("{}:{}", format_ip(&conn.remote_ip), conn.remote_port);
            let state = tcp_state_name(conn.state);

            let hostname = app
                .hostname_cache_arc
                .lock()
                .unwrap()
                .get(&conn.remote_ip)
                .unwrap_or(&String::new())
                .to_string();
            let state_style = match state {
                "ESTABLISHED" => Style::default().fg(Color::Rgb(100, 200, 100)),
                "LISTEN" => Style::default().fg(Color::Rgb(100, 150, 200)),
                "TIME_WAIT" => Style::default().fg(Color::Rgb(255, 200, 100)),
                "CLOSE_WAIT" => Style::default().fg(Color::Rgb(200, 150, 200)),
                "SYN_SENT" | "SYN_RECV" => Style::default().fg(Color::Rgb(150, 150, 200)),
                "FIN_WAIT1" | "FIN_WAIT2" => Style::default().fg(Color::Rgb(255, 220, 150)),
                _ => Style::default().fg(Color::Rgb(150, 150, 150)),
            };

            let queue_style = if conn.tx_queue > 0 || conn.rx_queue > 0 {
                Style::default()
                    .fg(Color::Rgb(255, 200, 100))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(80, 80, 80))
            };

            Row::new(vec![
                Cell::from(Span::styled(
                    format!(" {}", local_addr),
                    Style::default().fg(Color::Rgb(180, 180, 180)),
                )),
                Cell::from(Span::styled(
                    format!(" {}", remote_addr),
                    Style::default().fg(Color::Rgb(180, 180, 180)),
                )),
                Cell::from(Span::styled(
                    format!(" {}", hostname),
                    Style::default().fg(Color::Rgb(139, 233, 253)), // Cyan for hostname
                )),
                Cell::from(Span::styled(format!(" {}", state), state_style)),
                Cell::from(Span::styled(
                    format!(" {}:{}", conn.tx_queue, conn.rx_queue),
                    queue_style,
                )),
                Cell::from(Span::styled(
                    format!(" {}", conn.uid),
                    Style::default().fg(Color::Rgb(120, 120, 120)),
                )),
                Cell::from(Span::styled(
                    format!(" {}", conn.inode),
                    Style::default().fg(Color::Rgb(120, 120, 120)),
                )),
            ])
        })
        .collect();
    app.tcp_vertical_scroll_state = app.tcp_vertical_scroll_state.content_length(tcp_data.len());

    let visible_rows = (tcp_split[1].height as usize).saturating_sub(4);
    let scroll_offset = app.tcp_vertical_scroll;

    let visible_tcp_rows: Vec<Row> = tcp_rows
        .into_iter()
        .skip(scroll_offset)
        .take(visible_rows)
        .collect();

    let tcp_table = Table::new(
        visible_tcp_rows,
        [
            Constraint::Percentage(20), // Local Address (reduced)
            Constraint::Percentage(20), // Remote Address (reduced)
            Constraint::Percentage(25), // Hostname (NEW)
            Constraint::Length(12),     // State
            Constraint::Length(10),     // TX:RX
            Constraint::Length(6),      // UID
            Constraint::Fill(1),        // Inode
        ],
    )
    .header(
        Row::new(vec![
            Cell::from(" Local Address"),
            Cell::from(" Remote Address"),
            Cell::from(" Hostname"), // NEW COLUMN
            Cell::from(" State"),
            Cell::from(" TX:RX"),
            Cell::from(" UID"),
            Cell::from(" Inode"),
        ])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    )
    .block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Rgb(120, 120, 120)))
            .title(format!(" 🔌 TCP CONNECTIONS ({}) ", tcp_data.len()))
            .title_style(
                Style::default()
                    .fg(tcp_border_color)
                    .add_modifier(Modifier::BOLD),
            )
            .padding(ratatui::widgets::Padding {
                left: 1,
                right: 2,
                top: 0,
                bottom: 0,
            }),
    );

    frame.render_widget(tcp_table, tcp_split[1]);

    frame.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None)
            .track_symbol(None)
            .thumb_symbol("┃")
            .style(Style::default().fg(Color::Rgb(200, 200, 200))),
        tcp_split[1].inner(Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut app.tcp_vertical_scroll_state,
    );

    if app.show_help {
        render_help_popup(frame);
    }
    if app.enter_tick_active {
        draw_tick_mode(frame, app);
    }
}

fn render_help_popup(frame: &mut Frame) {
    let area = frame.area();

    let popup_area = centered_rect(70, 85, area);

    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Keys:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ↑ /↓        ", Style::default().fg(Color::Green)),
            Span::raw("Navigate interface list"),
        ]),
        Line::from(vec![
            Span::styled("    Enter           ", Style::default().fg(Color::Green)),
            Span::raw("Select interface"),
        ]),
        Line::from(vec![
            Span::styled("    Esc             ", Style::default().fg(Color::Green)),
            Span::raw("Clear selection / Exit filter"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Interface:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    f               ", Style::default().fg(Color::Green)),
            Span::raw("Filter interfaces by name"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Display:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    r               ", Style::default().fg(Color::Green)),
            Span::raw("Toggle raw bytes display"),
        ]),
        Line::from(vec![
            Span::styled("    d               ", Style::default().fg(Color::Green)),
            Span::raw("Decimal byte unit (KB, MB, GB)"),
        ]),
        Line::from(vec![
            Span::styled("    b               ", Style::default().fg(Color::Green)),
            Span::raw("Binary byte unit (KiB, MiB, GiB)"),
        ]),
        Line::from(vec![
            Span::styled("    K               ", Style::default().fg(Color::Green)),
            Span::raw("Update tick rate"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Other:",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("    ?, h            ", Style::default().fg(Color::Green)),
            Span::raw("Show/hide this help"),
        ]),
        Line::from(vec![
            Span::styled("    q               ", Style::default().fg(Color::Green)),
            Span::raw("Quit"),
        ]),
        Line::from(""),
        Line::from(vec![]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "?",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close", Style::default().fg(Color::DarkGray)),
        ]),
    ];

    let help = Paragraph::new(help_text).block(
        Block::bordered()
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Cyan))
            .title(vec![Span::raw(" "), Span::raw(" HELP ")])
            .title_style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    frame.render_widget(help, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

fn make_bar(percent: f64, width: usize) -> String {
    if width == 0 {
        return "".to_string();
    }
    let filled = (percent * width as f64) as usize;
    let filled = filled.min(width);
    let empty = width.saturating_sub(filled);
    format!("{}{}", "█".repeat(filled), "░".repeat(empty))
}

fn resolve_hostname(ip: &[u8; 4]) -> String {
    if ip == &[0, 0, 0, 0] || ip == &[127, 0, 0, 1] {
        return "-".to_string();
    }

    let ip_addr = IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3]));

    match dns_lookup::lookup_addr(&ip_addr) {
        std::result::Result::Ok(hostname) => {
            if hostname.len() > 30 {
                format!("{}...", &hostname[..27])
            } else {
                hostname
            }
        }
        Err(_) => "-".to_string(),
    }
}
