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
use ratatui::widgets::Cell;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
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

pub fn draw_interface_mode(
    app: &mut App,
    frame: &mut Frame,
    data: &Vec<NetworkStats>,
    tcp_data: &Vec<TcpStats>,
) {
    let byte_unit = app.byte_unit.clone();
    let area = frame.area();

    let chunks =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    let main_part = chunks[0];
    let tcp_area = chunks[1];

    let interface_count = data.len();
    let list_percentage = if interface_count <= 3 {
        35
    } else if interface_count <= 6 {
        30
    } else {
        25
    };

    let main_split = Layout::horizontal([
        Constraint::Percentage(list_percentage),
        Constraint::Percentage(100 - list_percentage),
    ])
    .split(main_part);

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

                    let rx_load = parse_speed_for_bar(&rx_speed_str);
                    let tx_load = parse_speed_for_bar(&tx_speed_str);
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

                    ListItem::new(vec![Line::from(vec![
                        Span::styled(
                            format!("{:>2}.", display_idx + 1),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::raw(" "),
                        Span::styled(format!("{:<16}", name), Style::default().fg(Color::Cyan)),
                        activity,
                    ])])
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

                    let rx_load = parse_speed_for_bar(&rx_speed_str);
                    let tx_load = parse_speed_for_bar(&tx_speed_str);
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

                    ListItem::new(vec![Line::from(vec![
                        Span::styled(
                            format!("{:>2}.", idx + 1),
                            Style::default().fg(Color::DarkGray),
                        ),
                        Span::raw(" "),
                        Span::styled(format!("{:<16}", name), Style::default().fg(Color::Cyan)),
                        activity,
                    ])])
                })
                .collect();

            let mut state = ListState::default();
            state.select(Some(app.vertical_scroll));

            let list = List::new(items).block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(format!(" 📡 INTERFACES ({}) ", interface_count))
                    .title_style(
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
            );

            frame.render_stateful_widget(list, list_area, &mut state);
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
                let rx_load = parse_speed_for_bar(&rx_speed_str);
                let tx_load = parse_speed_for_bar(&tx_speed_str);

                let detail_chunks = Layout::vertical([
                    Constraint::Length(5),
                    Constraint::Length(5),
                    Constraint::Fill(1),
                ])
                .split(detail_area);

                let rx_current = parse_speed_to_mbps(&rx_speed_str);
                let tx_current = parse_speed_to_mbps(&tx_speed_str);

                let rx_peak = app
                    .rx_peak_speed
                    .entry(selected_name.clone())
                    .or_insert(0.0);
                if rx_current > *rx_peak {
                    *rx_peak = rx_current;
                }
                let tx_peak = app
                    .tx_peak_speed
                    .entry(selected_name.clone())
                    .or_insert(0.0);
                if tx_current > *tx_peak {
                    *tx_peak = tx_current;
                }

                let rx_avg = app.rx_avg_speed.entry(selected_name.clone()).or_insert(0.0);
                let tx_avg = app.tx_avg_speed.entry(selected_name.clone()).or_insert(0.0);

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
                        .green(),
                );
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
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  Name:  ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            &interface_data.name,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Total: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(total_str, Style::default().fg(Color::Yellow)),
                    ]),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .title(" INFO ")
                        .cyan(),
                );

                let middle_col = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  Bytes:   ", Style::default().fg(Color::DarkGray)),
                        Span::styled(rx_bytes_str, Style::default().fg(Color::White)),
                    ]),
                    Line::from(vec![
                        Span::styled("  Packets: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.receive.packets),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Errors:  ", Style::default().fg(Color::DarkGray)),
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
                        Span::styled("  Drops:   ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.receive.drop),
                            if interface_data.receive.drop > 0 {
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            },
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
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  Bytes:   ", Style::default().fg(Color::DarkGray)),
                        Span::styled(tx_bytes_str, Style::default().fg(Color::White)),
                    ]),
                    Line::from(vec![
                        Span::styled("  Packets: ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.transmit.packets),
                            Style::default().fg(Color::White),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Errors:  ", Style::default().fg(Color::DarkGray)),
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
                        Span::styled("  Drops:   ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            format!("{}", interface_data.transmit.drop),
                            if interface_data.transmit.drop > 0 {
                                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                            } else {
                                Style::default().fg(Color::Green)
                            },
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
                Line::from(""),
                Line::from(Span::styled(
                    "  ALL INTERFACES",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("  Total RX:      ", Style::default().fg(Color::DarkGray)),
                    Span::styled(summary_rx_val, Style::default().fg(Color::Green)),
                ]),
                Line::from(vec![
                    Span::styled("  Total TX:      ", Style::default().fg(Color::DarkGray)),
                    Span::styled(summary_tx_val, Style::default().fg(Color::Blue)),
                ]),
                Line::from(vec![
                    Span::styled("  Total Packets: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        format!("{}", total_packets),
                        Style::default().fg(Color::White),
                    ),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "  💡 Select an interface for details",
                    Style::default().fg(Color::DarkGray).italic(),
                )),
            ])
            .block(
                Block::bordered()
                    .border_type(BorderType::Rounded)
                    .title(" 📊 OVERVIEW ")
                    .cyan(),
            )
            .alignment(Alignment::Left);

            frame.render_widget(summary, detail_area);
        }
    }

    let tcp_rows: Vec<Row> = tcp_data
        .iter()
        .map(|conn| {
            let local_addr = format!("{}:{}", format_ip(&conn.local_ip), conn.local_port);
            let remote_addr = format!("{}:{}", format_ip(&conn.remote_ip), conn.remote_port);
            let state = tcp_state_name(conn.state);
            let timer = format_timer(conn.timer_active);

            let state_style = match state {
                "ESTABLISHED" => Style::default().fg(Color::Green),
                "LISTEN" => Style::default().fg(Color::Cyan),
                "TIME_WAIT" => Style::default().fg(Color::Yellow),
                "CLOSE_WAIT" => Style::default().fg(Color::Magenta),
                "SYN_SENT" | "SYN_RECV" => Style::default().fg(Color::Blue),
                "FIN_WAIT1" | "FIN_WAIT2" => Style::default().fg(Color::LightYellow),
                _ => Style::default().fg(Color::White),
            };

            let queue_style = if conn.tx_queue > 0 || conn.rx_queue > 0 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let timer_style = if conn.timer_active > 0 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            Row::new(vec![
                Cell::from(format!("{}", conn.sl)),
                Cell::from(local_addr),
                Cell::from(remote_addr),
                Cell::from(Span::styled(state, state_style)),
                Cell::from(Span::styled(
                    format!("{}:{}", conn.tx_queue, conn.rx_queue),
                    queue_style,
                )),
                Cell::from(Span::styled(timer, timer_style)),
                Cell::from(format!("{}", conn.timer_when)),
                Cell::from(format!("{}", conn.retransmit_timeout)),
                Cell::from(format!("{}", conn.uid)),
                Cell::from(format!("{}", conn.timeout)),
                Cell::from(format!("{}", conn.inode)),
            ])
        })
        .collect();

    let tcp_table = Table::new(
        tcp_rows,
        [
            Constraint::Length(4),      // sl
            Constraint::Percentage(20), // local_address
            Constraint::Percentage(20), // remote_address
            Constraint::Length(15),     // state
            Constraint::Length(12),     // tx:rx
            Constraint::Length(10),     // timer
            Constraint::Length(10),     // when
            Constraint::Length(10),     // retrnsmt
            Constraint::Length(10),     // uid
            Constraint::Length(10),     // timeout
            Constraint::Fill(1),        // inode
        ],
    )
    .header(
        Row::new(vec![
            Cell::from("Sl"),
            Cell::from("Local Address"),
            Cell::from("Remote Address"),
            Cell::from("State"),
            Cell::from("TX:RX"),
            Cell::from("Timer"),
            Cell::from("When"),
            Cell::from("Retrns"),
            Cell::from("UID"),
            Cell::from("Timeout"),
            Cell::from("Inode"),
        ])
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1),
    )
    .block(
        Block::bordered()
            .border_type(BorderType::Rounded)
            .title(format!(" 🔌 TCP CONNECTIONS ({}) ", tcp_data.len()))
            .title_style(
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
    );

    frame.render_widget(tcp_table, tcp_area);
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

fn extract_speed(data_str: &str) -> String {
    data_str
        .split("speed: ")
        .nth(1)
        .unwrap_or("0 B/s")
        .trim()
        .to_string()
}

fn extract_speed_from_line(line: &Line) -> String {
    let text: String = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect();
    extract_speed(&text)
}

fn parse_speed_for_bar(speed_str: &str) -> f64 {
    let link_speed_mbps = 5.0;

    if speed_str.contains("GB/s") {
        let val: f64 = speed_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        (val * 1000.0 / link_speed_mbps).min(1.0)
    } else if speed_str.contains("MB/s") {
        let val: f64 = speed_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        (val / link_speed_mbps).min(1.0)
    } else if speed_str.contains("KB/s") {
        let val: f64 = speed_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);
        (val / 1000.0 / link_speed_mbps).min(1.0)
    } else {
        0.0
    }
}
fn parse_speed_to_mbps(speed_str: &str) -> f64 {
    if speed_str.contains("GB/s") {
        speed_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0)
            * 1000.0
    } else if speed_str.contains("MB/s") {
        speed_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0)
    } else if speed_str.contains("KB/s") {
        speed_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0)
            / 1000.0
    } else {
        0.0
    }
}
fn format_ip(ip: &[u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

fn tcp_state_name(state: u64) -> &'static str {
    match state {
        0x01 => "ESTABLISHED",
        0x02 => "SYN_SENT",
        0x03 => "SYN_RECV",
        0x04 => "FIN_WAIT1",
        0x05 => "FIN_WAIT2",
        0x06 => "TIME_WAIT",
        0x07 => "CLOSE",
        0x08 => "CLOSE_WAIT",
        0x09 => "LAST_ACK",
        0x0A => "LISTEN",
        0x0B => "CLOSING",
        _ => "UNKNOWN",
    }
}

fn format_timer(timer_active: u64) -> &'static str {
    match timer_active {
        0 => "off",
        1 => "on",
        2 => "keepalive",
        3 => "timewait",
        4 => "probe",
        _ => "unknown",
    }
}
fn format_speed_mbps(mbps: f64) -> String {
    if mbps >= 1000.0 {
        format!("{:.2} GB/s", mbps / 1000.0)
    } else if mbps >= 1.0 {
        format!("{:.2} MB/s", mbps)
    } else {
        format!("{:.2} KB/s", mbps * 1000.0)
    }
}
