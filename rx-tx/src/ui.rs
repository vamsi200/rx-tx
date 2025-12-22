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
use ratatui::widgets::RenderDirection;
use ratatui::widgets::Row;
use ratatui::widgets::Sparkline;
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
use std::collections::HashSet;
use std::fmt::format;
use std::net::IpAddr;
use std::net::Ipv4Addr;
use std::ops::Sub;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use std::vec;

#[derive(Debug, Default, Clone, Copy)]
pub struct NetTotals {
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub total_bytes: u64,

    pub total_rx_packets: u64,
    pub total_tx_packets: u64,
    pub total_packets: u64,

    pub rx_tx_bytes_ratio: f64,
    pub rx_tx_packets_ratio: f64,

    pub total_rx_errors: u64,
    pub total_tx_errors: u64,
    pub total_errors: u64,

    pub total_rx_drops: u64,
    pub total_tx_drops: u64,
    pub total_drops: u64,

    pub error_rate_pct: f64,
    pub drop_rate_pct: f64,
}

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
                .border_type(BorderType::Plain)
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

pub fn compute_totals(data: &[NetworkStats]) -> NetTotals {
    let mut totals = NetTotals::default();

    for iface in data {
        totals.total_rx_bytes += iface.receive.bytes;
        totals.total_tx_bytes += iface.transmit.bytes;

        totals.total_rx_packets += iface.receive.packets;
        totals.total_tx_packets += iface.transmit.packets;

        totals.total_rx_errors += iface.receive.errs;
        totals.total_tx_errors += iface.transmit.errs;

        totals.total_rx_drops += iface.receive.drop;
        totals.total_tx_drops += iface.transmit.drop;
    }

    totals.total_bytes = totals.total_rx_bytes + totals.total_tx_bytes;
    totals.total_packets = totals.total_rx_packets + totals.total_tx_packets;

    totals.total_errors = totals.total_rx_errors + totals.total_tx_errors;
    totals.total_drops = totals.total_rx_drops + totals.total_tx_drops;

    if totals.total_tx_bytes > 0 {
        totals.rx_tx_bytes_ratio = totals.total_rx_bytes as f64 / totals.total_tx_bytes as f64;
    }

    if totals.total_tx_packets > 0 {
        totals.rx_tx_packets_ratio =
            totals.total_rx_packets as f64 / totals.total_tx_packets as f64;
    }

    if totals.total_packets > 0 {
        totals.error_rate_pct = totals.total_errors as f64 * 100.0 / totals.total_packets as f64;
        totals.drop_rate_pct = totals.total_drops as f64 * 100.0 / totals.total_packets as f64;
    }

    totals
}

fn render_overview_graph(frame: &mut Frame, area: Rect, app: &App) {
    let rows =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).split(area);

    let rx_data = &app.total_rx_history;
    let tx_data = &app.total_tx_history;

    let rx_spark = Sparkline::default()
        .block(
            Block::bordered()
                .title(" RX (MB/s) ")
                .title_alignment(ratatui::layout::Alignment::Left)
                .style(Style::default().fg(Color::Green)),
        )
        .data(rx_data)
        .style(Style::default().fg(Color::Green))
        .max(rx_data.iter().copied().max().unwrap_or(0))
        .direction(RenderDirection::LeftToRight)
        .absent_value_symbol(symbols::line::HORIZONTAL);

    let tx_spark = Sparkline::default()
        .block(
            Block::bordered()
                .title(" TX (MB/s) ")
                .title_alignment(ratatui::layout::Alignment::Left)
                .style(Style::default().fg(Color::Blue)),
        )
        .data(tx_data)
        .style(Style::default().fg(Color::Blue))
        .max(tx_data.iter().copied().max().unwrap_or(0))
        .direction(RenderDirection::LeftToRight)
        .absent_value_symbol(symbols::line::HORIZONTAL);

    frame.render_widget(rx_spark, rows[0]);
    frame.render_widget(tx_spark, rows[1]);
}

pub fn draw_speed_edit_popup(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let popup_area = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Length(8),
        Constraint::Percentage(40),
    ])
    .split(area)[1];

    let popup_area = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
    ])
    .split(popup_area)[1];

    let (title, field_name, color) = if app.edit_rx_mode {
        (" ⬇ Set Download Speed ", "Download (Mbps)", Color::Green)
    } else {
        (" ⬆ Set Upload Speed ", "Upload (Mbps)", Color::Blue)
    };

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            format!(
                "  Interface: {}",
                app.editing_interface.as_ref().unwrap_or(&"".to_string())
            ),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!("  {}: ", field_name), Style::default().fg(color)),
            Span::styled(
                &app.speed_input,
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "  Enter: Save | Esc: Cancel",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let popup = Paragraph::new(text)
        .block(
            Block::bordered()
                .border_type(BorderType::Plain)
                .border_style(Style::default().fg(color))
                .title(title)
                .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(Clear, popup_area);
    frame.render_widget(popup, popup_area);
}

pub fn tcp_matches_filter(
    conn: &TcpStats,
    filter: &str,
    hostname_cache: &HashMap<[u8; 4], String>,
) -> bool {
    if filter.is_empty() {
        return true;
    }

    let filter_lower = filter.to_lowercase();

    let local_addr = format!("{}:{}", format_ip(&conn.local_ip), conn.local_port);
    if local_addr.to_lowercase().contains(&filter_lower) {
        return true;
    }

    let remote_addr = format!("{}:{}", format_ip(&conn.remote_ip), conn.remote_port);
    if remote_addr.to_lowercase().contains(&filter_lower) {
        return true;
    }

    if let Some(hostname) = hostname_cache.get(&conn.remote_ip) {
        if hostname.to_lowercase().contains(&filter_lower) {
            return true;
        }
    }

    let state = tcp_state_name(conn.state);
    if state.to_lowercase().contains(&filter_lower) {
        return true;
    }

    if format!("{}", conn.uid).contains(&filter_lower) {
        return true;
    }

    if format!("{}", conn.inode).contains(&filter_lower) {
        return true;
    }

    false
}

struct Theme {
    system_uptime: Color,
    system_uptime_data: Color,
    total_rx: Color,
    total_rx_data: Color,
    total_tx: Color,
    total_tx_data: Color,
    total_packets: Color,
    total_packets_data: Color,
    total_error: Color,
    total_error_data: Color,
    total_drop: Color,
    total_drop_data: Color,
    total_error_ratio: Color,
    total_error_ratio_data: Color,
    total_drop_ratio: Color,
    total_drop_ratio_data: Color,
    total_rxtx_bytes_ratio: Color,
    total_rxtx_bytes_ratio_data: Color,
    total_rxtx_packets: Color,
    total_rxtx_packets_data: Color,

    tcp_info_connections: Color,
    tcp_info_total: Color,
    tcp_info_total_data: Color,

    tcp_info_active: Color,
    tcp_info_active_data: Color,

    tcp_info_unique_ip: Color,
    tcp_info_unique_ip_data: Color,

    tcp_info_localext: Color,
    tcp_info_localext_data: Color,

    tcp_info_established: Color,
    tcp_info_established_data: Color,

    tcp_info_listen: Color,
    tcp_info_listen_data: Color,

    tcp_info_common: Color,
    tcp_info_common_data: Color,

    interface_border: Color,
    tcp_border: Color,
    activity_symbol: Color,
    interface_index: Color,
    interface_name: Color,
    filter: Color,
    filter_highlight_symbol: Color,
    rx_bar: Color,
    tx_bar: Color,
    rx_cur_speed: Color,
    rx_peak_speed: Color,
    rx_avg_speed: Color,
    rx_link_speed: Color,
    tick: Color,
    tx_cur_speed: Color,
    tx_peak_speed: Color,
    tx_avg_speed: Color,
    tx_link_speed: Color,
    info_heading: Color,
    info_name: Color,
    info_name_data: Color,
    info_total: Color,
    info_total_data: Color,
    info_rx_bytes: Color,
    info_rx_bytes_data: Color,
    info_rx_packets: Color,
    info_rx_packets_data: Color,
    info_tx_bytes: Color,
    info_tx_bytes_data: Color,
    info_tx_packets: Color,
    info_tx_packets_data: Color,
    rx_error: Color,
    rx_error_data: Color,
    rx_drops: Color,
    rx_drops_data: Color,
    rx_fifo: Color,
    rx_fifo_data: Color,
    rx_frame: Color,
    rx_frame_data: Color,
    rx_compressed: Color,
    rx_compressed_data: Color,
    rx_multicast: Color,
    rx_multicast_data: Color,
    tx_error: Color,
    tx_error_data: Color,
    tx_drops: Color,
    tx_drops_data: Color,
    tx_fifo: Color,
    tx_fifo_data: Color,
    tx_collisions: Color,
    tx_collisions_data: Color,
    tx_carrier: Color,
    tx_carrier_data: Color,
    tx_compressed: Color,
    tx_compressed_data: Color,
    local_addr_data: Color,
    remote_addr_data: Color,
    hostname_data: Color,
    state_data: Color,
    tx_rx_data: Color,
    uid_data: Color,
    inode_data: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            system_uptime: Color::DarkGray,
            system_uptime_data: Color::White,

            total_rx: Color::DarkGray,
            total_rx_data: Color::White,

            total_tx: Color::DarkGray,
            total_tx_data: Color::White,

            total_packets: Color::DarkGray,
            total_packets_data: Color::White,

            total_error: Color::DarkGray,
            total_error_data: Color::White,

            total_drop: Color::DarkGray,
            total_drop_data: Color::White,

            total_error_ratio: Color::DarkGray,
            total_error_ratio_data: Color::White,

            total_drop_ratio: Color::DarkGray,
            total_drop_ratio_data: Color::White,

            total_rxtx_bytes_ratio: Color::DarkGray,
            total_rxtx_bytes_ratio_data: Color::White,

            total_rxtx_packets: Color::DarkGray,
            total_rxtx_packets_data: Color::White,

            tcp_info_connections: Color::Yellow,

            tcp_info_total: Color::DarkGray,
            tcp_info_total_data: Color::Green,

            tcp_info_active: Color::DarkGray,
            tcp_info_active_data: Color::White,

            tcp_info_unique_ip: Color::DarkGray,
            tcp_info_unique_ip_data: Color::White,

            tcp_info_localext: Color::DarkGray,
            tcp_info_localext_data: Color::White,

            tcp_info_established: Color::DarkGray,
            tcp_info_established_data: Color::White,

            tcp_info_listen: Color::DarkGray,
            tcp_info_listen_data: Color::White,

            tcp_info_common: Color::DarkGray,
            tcp_info_common_data: Color::White,

            interface_border: Color::White,
            tcp_border: Color::White,

            interface_index: Color::DarkGray,
            interface_name: Color::White,
            activity_symbol: Color::Green,

            filter: Color::Yellow,
            filter_highlight_symbol: Color::Yellow,

            rx_bar: Color::Green,
            rx_cur_speed: Color::DarkGray,
            rx_peak_speed: Color::DarkGray,
            rx_avg_speed: Color::DarkGray,
            rx_link_speed: Color::DarkGray,

            tx_bar: Color::Blue,
            tx_cur_speed: Color::DarkGray,
            tx_peak_speed: Color::DarkGray,
            tx_avg_speed: Color::DarkGray,
            tx_link_speed: Color::DarkGray,

            tick: Color::DarkGray,

            info_heading: Color::LightYellow,
            info_name: Color::DarkGray,
            info_name_data: Color::White,
            info_total: Color::DarkGray,
            info_total_data: Color::White,
            info_rx_bytes: Color::DarkGray,
            info_rx_bytes_data: Color::White,
            info_rx_packets: Color::DarkGray,
            info_rx_packets_data: Color::White,
            info_tx_bytes: Color::DarkGray,
            info_tx_bytes_data: Color::White,
            info_tx_packets: Color::DarkGray,
            info_tx_packets_data: Color::White,

            rx_error: Color::DarkGray,
            rx_error_data: Color::White,
            rx_drops: Color::DarkGray,
            rx_drops_data: Color::White,
            rx_fifo: Color::DarkGray,
            rx_fifo_data: Color::White,
            rx_frame: Color::DarkGray,
            rx_frame_data: Color::White,
            rx_compressed: Color::DarkGray,
            rx_compressed_data: Color::White,
            rx_multicast: Color::DarkGray,
            rx_multicast_data: Color::White,

            tx_error: Color::DarkGray,
            tx_error_data: Color::White,
            tx_drops: Color::DarkGray,
            tx_drops_data: Color::White,
            tx_fifo: Color::DarkGray,
            tx_fifo_data: Color::White,
            tx_collisions: Color::DarkGray,
            tx_collisions_data: Color::White,
            tx_carrier: Color::DarkGray,
            tx_carrier_data: Color::White,
            tx_compressed: Color::DarkGray,
            tx_compressed_data: Color::White,

            local_addr_data: Color::Green,
            remote_addr_data: Color::Blue,
            hostname_data: Color::Yellow,
            state_data: Color::White,
            tx_rx_data: Color::White,
            uid_data: Color::White,
            inode_data: Color::White,
        }
    }
}

pub fn draw_interface_mode(
    app: &mut App,
    frame: &mut Frame,
    data: &Vec<NetworkStats>,
    tcp_data: &Vec<TcpStats>,
    rx_peak_speed: &mut HashMap<String, f64>,
    tx_peak_speed: &mut HashMap<String, f64>,
    rx_avg_speed: &mut HashMap<String, f64>,
    tx_avg_speed: &mut HashMap<String, f64>,
) {
    let theme = Theme::default();

    let interface_border = if app.focus == Focus::Interfaces {
        Color::Red
    } else {
        theme.interface_border
    };

    let tcp_border = if app.focus == Focus::TcpTable {
        Color::Red
    } else {
        theme.interface_border
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

                    let rx_load = parse_speed(&rx_speed_str, None);
                    let tx_load = parse_speed(&tx_speed_str, None);

                    let is_active = rx_load > 0.01 || tx_load > 0.01;

                    let activity = if is_active {
                        Span::styled(
                            "⚡",
                            Style::default()
                                .fg(theme.activity_symbol)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw(" ")
                    };

                    ListItem::new(vec![Line::from(vec![
                        Span::raw(" "),
                        Span::styled(
                            format!("{:<16}", name),
                            Style::default().fg(theme.interface_name),
                        ),
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
                        .border_type(BorderType::Plain)
                        .title(format!(" Filter: {} ", filter))
                        .title_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                )
                .highlight_symbol("> ")
                .highlight_style(
                    Style::default()
                        .fg(theme.filter_highlight_symbol)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_spacing(HighlightSpacing::Always)
                .fg(Color::Red);

            frame.render_stateful_widget(list, list_area, &mut state);

            let mut scrollbar_state = ScrollbarState::new(filtered.len()).position(*index);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(None)
                    .thumb_symbol("┃"),
                list_area.inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
        _ => {
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

                    let rx_load = parse_speed(&rx_speed_str, None);
                    let tx_load = parse_speed(&tx_speed_str, None);
                    let is_active = rx_load > 0.01 || tx_load > 0.01;

                    let activity = if is_active {
                        Span::styled(
                            "⚡",
                            Style::default()
                                .fg(theme.activity_symbol)
                                .add_modifier(Modifier::BOLD),
                        )
                    } else {
                        Span::raw(" ")
                    };
                    ListItem::new(vec![Line::from(vec![
                        Span::raw(" "),
                        Span::styled(
                            format!("{:<16}", name),
                            Style::default().fg(theme.interface_name),
                        ),
                        activity,
                    ])])
                })
                .collect();

            let interface_count = interface_names.iter().count();
            let mut state = ListState::default();
            state.select(Some(app.vertical_scroll));

            let title = Line::from(vec![
                Span::styled(
                    " [f]",
                    Style::default()
                        .fg(Color::LightYellow)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" Interfaces ({interface_count})")),
            ]);

            let list = List::new(items).block(
                Block::bordered()
                    .border_type(BorderType::Plain)
                    .title(title)
                    .fg(interface_border),
            );

            frame.render_stateful_widget(list, list_area, &mut state);
            frame.render_stateful_widget(
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
                    *rx_peak = rx_current
                };

                let tx_peak = tx_peak_speed.entry(selected_name.clone()).or_insert(0.0);
                if tx_current > *tx_peak {
                    *tx_peak = tx_current;
                }

                let rx_avg_ref = rx_avg_speed.entry(selected_name.clone()).or_insert(0.0);
                let tx_avg_ref = tx_avg_speed.entry(selected_name.clone()).or_insert(0.0);

                if app.update_avg {
                    app.update_avg = false;
                    *rx_avg_ref = *rx_avg_ref * 0.95 + rx_current * 0.05;
                    *tx_avg_ref = *tx_avg_ref * 0.95 + tx_current * 0.05;
                }

                let rx_avg = *rx_avg_ref;
                let tx_avg = *tx_avg_ref;

                let rx_speed = app.get_rx_limit(&interface_data.name);
                let tx_speed = app.get_tx_limit(&interface_data.name);

                let rx_load = parse_speed(&rx_speed_str, Some(rx_speed));
                let tx_load = parse_speed(&tx_speed_str, Some(tx_speed));

                let rx_peak_str = format_speed_mbps(*rx_peak);
                let rx_avg_str = format_speed_mbps(rx_avg);
                let tx_peak_str = format_speed_mbps(*tx_peak);
                let tx_avg_str = format_speed_mbps(tx_avg);

                let rx_para = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            make_bar(rx_load, bar_width),
                            Style::default().fg(theme.rx_bar),
                        ),
                        Span::raw(" ▼"),
                    ]),
                    Line::from(""),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Plain)
                        .title(vec![
                            Span::styled(
                                " RX",
                                Style::default()
                                    .fg(theme.rx_bar)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Cur: ", Style::default().fg(theme.rx_cur_speed)),
                            Span::styled(
                                format!("{:<10}", rx_speed_str),
                                Style::default()
                                    .fg(theme.tx_bar)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Peak: ", Style::default().fg(theme.rx_peak_speed)),
                            Span::styled(
                                format!("{:<10}", rx_peak_str),
                                Style::default()
                                    .fg(theme.rx_bar)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Avg: ", Style::default().fg(theme.rx_avg_speed)),
                            Span::styled(
                                rx_avg_str,
                                Style::default()
                                    .fg(theme.tx_bar)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" "),
                        ])
                        .title_top(
                            Line::from(vec![
                                Span::styled(
                                    " [R] Link Speed: ",
                                    Style::default().fg(theme.rx_link_speed),
                                ),
                                Span::styled(
                                    format!(
                                        "{} Mbps",
                                        app.interface_speeds
                                            .get(&selected_name)
                                            .map(|(rx, _)| format!("{:.0}", rx))
                                            .unwrap_or("?".to_string())
                                    ),
                                    Style::default()
                                        .fg(theme.rx_bar)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(" │ "),
                                Span::styled("Tick: ", Style::default().fg(theme.tick)),
                                Span::styled(
                                    tick_display.clone(),
                                    Style::default()
                                        .fg(theme.tx_bar)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::styled(" (K) ", Style::default().fg(theme.tick)),
                            ])
                            .right_aligned(),
                        ),
                )
                .fg(Color::Green)
                .alignment(Alignment::Left);

                frame.render_widget(rx_para, detail_chunks[0]);

                let tx_para = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::raw("  "),
                        Span::styled(
                            make_bar(tx_load, bar_width),
                            Style::default().fg(theme.tx_bar),
                        ),
                        Span::raw(" ▲"),
                    ]),
                    Line::from(""),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Plain)
                        .title(vec![
                            Span::styled(
                                " TX",
                                Style::default()
                                    .fg(theme.tx_bar)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Cur: ", Style::default().fg(theme.tx_cur_speed)),
                            Span::styled(
                                format!("{:<10}", tx_speed_str),
                                Style::default()
                                    .fg(theme.rx_bar)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Peak: ", Style::default().fg(theme.tx_peak_speed)),
                            Span::styled(
                                format!("{:<10}", tx_peak_str),
                                Style::default()
                                    .fg(theme.tx_bar)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled("Avg: ", Style::default().fg(theme.tx_avg_speed)),
                            Span::styled(
                                tx_avg_str,
                                Style::default()
                                    .fg(theme.rx_bar)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" "),
                        ])
                        .border_style(Style::default().fg(Color::Blue))
                        .title_top(
                            Line::from(vec![
                                Span::styled(
                                    " [T] Link Speed: ",
                                    Style::default().fg(theme.tx_link_speed),
                                ),
                                Span::styled(
                                    format!(
                                        "{} Mbps",
                                        app.interface_speeds
                                            .get(&selected_name)
                                            .map(|(_, tx)| format!("{:.0}", tx))
                                            .unwrap_or("?".to_string())
                                    ),
                                    Style::default()
                                        .fg(theme.tx_bar)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(" "),
                            ])
                            .right_aligned(),
                        ),
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
                        Span::styled("  Name        : ", Style::default().fg(theme.info_name)),
                        Span::styled(
                            &interface_data.name,
                            Style::default()
                                .fg(theme.info_name_data)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Total       : ", Style::default().fg(theme.info_total)),
                        Span::styled(total_str, Style::default().fg(theme.info_total_data)),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  RX Bytes    : ", Style::default().fg(theme.info_rx_bytes)),
                        Span::styled(rx_bytes_str, Style::default().fg(theme.info_rx_bytes_data)),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  RX Packets  : ",
                            Style::default().fg(theme.info_rx_packets),
                        ),
                        Span::styled(
                            format!("{}", interface_data.receive.packets),
                            Style::default().fg(theme.info_rx_packets_data),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  TX Bytes    : ", Style::default().fg(theme.info_tx_bytes)),
                        Span::styled(tx_bytes_str, Style::default().fg(theme.info_tx_bytes_data)),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  TX Packets  : ",
                            Style::default().fg(theme.info_tx_packets),
                        ),
                        Span::styled(
                            format!("{}", interface_data.transmit.packets),
                            Style::default().fg(theme.info_tx_packets_data),
                        ),
                    ]),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Plain)
                        .title(" INFO ")
                        .title_style(Style::new().bold())
                        .border_style(Style::default().fg(theme.info_heading)),
                );

                let middle_col = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled("  Errors      : ", Style::default().fg(theme.rx_error)),
                        Span::styled(
                            format!("{}", interface_data.receive.errs),
                            Style::default().fg(theme.rx_error_data),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Drops       : ", Style::default().fg(theme.rx_drops)),
                        Span::styled(
                            format!("{}", interface_data.receive.drop),
                            Style::default().fg(theme.rx_drops_data),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  FIFO        : ", Style::default().fg(theme.rx_fifo)),
                        Span::styled(
                            format!("{}", interface_data.receive.fifo),
                            Style::default().fg(theme.rx_fifo_data),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Frame       : ", Style::default().fg(theme.rx_frame)),
                        Span::styled(
                            format!("{}", interface_data.receive.frame),
                            Style::default().fg(theme.rx_frame_data),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  Compressed  : ", Style::default().fg(theme.rx_compressed)),
                        Span::styled(
                            format!("{}", interface_data.receive.compressed),
                            Style::default().fg(theme.rx_compressed_data),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Multicast   : ", Style::default().fg(theme.rx_multicast)),
                        Span::styled(
                            format!("{}", interface_data.receive.multicast),
                            Style::default().fg(theme.rx_multicast_data),
                        ),
                    ]),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Plain)
                        .title(" RX ")
                        .title_style(Style::new().bold())
                        .border_style(Style::default().fg(theme.rx_bar)),
                );

                let right_col = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled("  Errors       : ", Style::default().fg(theme.tx_error)),
                        Span::styled(
                            format!("{}", interface_data.transmit.errs),
                            Style::default().fg(theme.tx_error_data),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("  Drops        : ", Style::default().fg(theme.tx_drops)),
                        Span::styled(
                            format!("{}", interface_data.transmit.drop),
                            Style::default().fg(theme.tx_drops_data),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  FIFO         : ", Style::default().fg(theme.tx_fifo)),
                        Span::styled(
                            format!("{}", interface_data.transmit.fifo),
                            Style::default().fg(theme.tx_fifo_data),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  Collisions   : ",
                            Style::default().fg(theme.tx_collisions),
                        ),
                        Span::styled(
                            format!("{}", interface_data.transmit.colls),
                            Style::default().fg(theme.tx_collisions_data),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled("  Carrier      : ", Style::default().fg(theme.tx_carrier)),
                        Span::styled(
                            format!("{}", interface_data.transmit.carrier),
                            Style::default().fg(theme.tx_carrier_data),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            "  Compressed   : ",
                            Style::default().fg(theme.tx_compressed),
                        ),
                        Span::styled(
                            format!("{}", interface_data.transmit.compressed),
                            Style::default().fg(theme.tx_compressed_data),
                        ),
                    ]),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Plain)
                        .title(" TX ")
                        .title_style(Style::new().bold())
                        .border_style(Style::default().fg(theme.tx_bar)),
                );

                frame.render_widget(left_col, stats_columns[0]);
                frame.render_widget(middle_col, stats_columns[1]);
                frame.render_widget(right_col, stats_columns[2]);
            }
        }
        InterfaceSelected::All => {
            let cols = Layout::horizontal([Constraint::Percentage(45), Constraint::Fill(1)])
                .split(detail_area);

            let left_area = cols[0];
            let right_area = cols[1];

            let totals = compute_totals(data);

            let summary_rx_val = if app.raw_bytes {
                totals.total_rx_bytes.to_string()
            } else {
                format_bytes(totals.total_rx_bytes, &byte_unit)
            };
            let summary_tx_val = if app.raw_bytes {
                totals.total_tx_bytes.to_string()
            } else {
                format_bytes(totals.total_tx_bytes, &byte_unit)
            };

            let summary = Paragraph::new(vec![
                Line::from(vec![
                    Span::styled(
                        "  System Uptime       : ",
                        Style::default().fg(theme.system_uptime),
                    ),
                    Span::styled(uptime, Style::default().fg(theme.system_uptime_data)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  Total RX            : ",
                        Style::default().fg(theme.total_rx),
                    ),
                    Span::styled(summary_rx_val, Style::default().fg(theme.total_rx_data)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  Total TX            : ",
                        Style::default().fg(theme.total_tx),
                    ),
                    Span::styled(summary_tx_val, Style::default().fg(theme.total_tx_data)),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  Total Packets       : ",
                        Style::default().fg(theme.total_packets),
                    ),
                    Span::styled(
                        format!("{}", totals.total_packets),
                        Style::default().fg(theme.total_packets_data),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  Total Errors        : ",
                        Style::default().fg(theme.total_error),
                    ),
                    Span::styled(
                        format!("{}", totals.total_errors),
                        if totals.total_errors > 0 {
                            Style::default()
                                .fg(theme.total_tx_data)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(theme.total_error_data)
                        },
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  Total Drops         : ",
                        Style::default().fg(theme.total_drop_ratio),
                    ),
                    Span::styled(
                        format!("{}", totals.total_drops),
                        Style::default().fg(theme.total_drop_ratio_data),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  Error Rate Ratio    : ",
                        Style::default().fg(theme.total_error_ratio),
                    ),
                    Span::styled(
                        if totals.error_rate_pct > 0.0 {
                            format!("{:.3}%", totals.error_rate_pct)
                        } else {
                            "-".to_string()
                        },
                        Style::default().fg(theme.total_error_ratio_data),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  Drop Rate Ratio     : ",
                        Style::default().fg(theme.total_drop_ratio),
                    ),
                    Span::styled(
                        if totals.drop_rate_pct > 0.0 {
                            format!("{:.3}%", totals.drop_rate_pct)
                        } else {
                            "-".to_string()
                        },
                        Style::default().fg(theme.total_drop_ratio_data),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  RX/TX Bytes Ratio   : ",
                        Style::default().fg(theme.total_rxtx_bytes_ratio),
                    ),
                    Span::styled(
                        if totals.rx_tx_bytes_ratio > 0.0 {
                            format!("{:.2}", totals.rx_tx_bytes_ratio)
                        } else {
                            "-".to_string()
                        },
                        Style::default().fg(theme.total_rxtx_bytes_ratio_data),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        "  RX/TX Packets Ratio : ",
                        Style::default().fg(theme.total_rxtx_packets),
                    ),
                    Span::styled(
                        if totals.rx_tx_packets_ratio > 0.0 {
                            format!("{:.2}", totals.rx_tx_packets_ratio)
                        } else {
                            "-".to_string()
                        },
                        Style::default().fg(theme.total_rxtx_packets_data),
                    ),
                ]),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(vec![Span::styled(
                    " Press `h` or `?` for help",
                    Style::default().fg(theme.interface_name),
                )]),
            ])
            .block(
                Block::bordered()
                    .border_type(BorderType::Plain)
                    .title_top(Line::from(" OVERVIEW ").left_aligned())
                    .title_top(
                        Line::from(vec![
                            Span::styled(" Tick: ", Style::default().fg(theme.tick)),
                            Span::styled(
                                tick_display,
                                Style::default()
                                    .fg(theme.info_rx_bytes)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(" (K) ", Style::default().fg(theme.tick)),
                        ])
                        .right_aligned(),
                    )
                    .border_style(Style::new().fg(Color::Rgb(115, 194, 251))),
            )
            .alignment(Alignment::Left);
            frame.render_widget(summary, left_area);
            render_overview_graph(frame, right_area, app);
        }
    }

    let tcp_split =
        Layout::horizontal([Constraint::Length(25), Constraint::Fill(1)]).split(tcp_area);

    match &app.mode {
        Mode::FilterLocalAddress { filter, index } => {
            let hostname_cache = app.hostname_cache_arc.lock().unwrap();
            let mut filtered_tcp: Vec<&TcpStats> = tcp_data
                .iter()
                .filter(|conn| tcp_matches_filter(conn, filter, &hostname_cache))
                .collect();

            let display_index = if let Some(selected_idx) = app.selected_index {
                if selected_idx < filtered_tcp.len() {
                    filtered_tcp = vec![filtered_tcp[selected_idx]];
                    0
                } else {
                    app.selected_index = None;
                    *index
                }
            } else {
                *index
            };

            let mut state_counts: std::collections::BTreeMap<&str, usize> =
                std::collections::BTreeMap::new();

            for conn in filtered_tcp.iter() {
                let state = tcp_state_name(conn.state);
                *state_counts.entry(state).or_insert(0) += 1;

                let ip = conn.remote_ip;
                if ip == [0, 0, 0, 0] || ip == [127, 0, 0, 1] {
                    continue;
                }

                let needs_lookup = !hostname_cache.contains_key(&ip);
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

                    let hostname_cache = app.hostname_cache_arc.lock().unwrap();
                }
            }
            let mut summary_lines = vec![
                Line::from("  Connections").style(Style::default().fg(theme.tcp_info_connections)),
                Line::from(vec![
                    Span::styled(
                        "  Total       : ",
                        Style::default().fg(theme.interface_index),
                    ),
                    Span::styled(
                        format!("{}", tcp_data.len()),
                        Style::default()
                            .fg(theme.interface_name)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
            ];

            let unique_ips: HashSet<[u8; 4]> = tcp_data
                .iter()
                .filter(|c| c.remote_ip != [0, 0, 0, 0] && c.remote_ip != [127, 0, 0, 1])
                .map(|c| c.remote_ip)
                .collect();

            let active = tcp_data
                .iter()
                .filter(|c| c.tx_queue > 0 || c.rx_queue > 0)
                .count();
            let local_only = tcp_data
                .iter()
                .filter(|c| c.remote_ip == [127, 0, 0, 1] || c.remote_ip == [0, 0, 0, 0])
                .count();

            summary_lines.push(Line::from(vec![
                Span::styled(
                    "  Active      : ",
                    Style::default().fg(theme.tcp_info_active),
                ),
                Span::styled(
                    format!("{}", active),
                    Style::default().fg(theme.tcp_info_active_data),
                ),
            ]));

            summary_lines.push(Line::from(vec![
                Span::styled(
                    "  Unique IPs  : ",
                    Style::default().fg(theme.tcp_info_unique_ip),
                ),
                Span::styled(
                    format!("{}", unique_ips.len()),
                    Style::default().fg(theme.tcp_info_unique_ip_data),
                ),
            ]));

            summary_lines.push(Line::from(vec![
                Span::styled(
                    "  Local/Ext   : ",
                    Style::default().fg(theme.tcp_info_localext),
                ),
                Span::styled(
                    format!("{}", local_only),
                    Style::default().fg(theme.tcp_info_localext_data),
                ),
                Span::raw("/"),
                Span::styled(
                    format!("{}", tcp_data.len() - local_only),
                    Style::default().fg(theme.tcp_info_localext_data),
                ),
            ]));

            summary_lines.push(Line::from(""));
            for (state, count) in state_counts.iter() {
                let color = match *state {
                    "ESTABLISHED" => theme.tcp_info_established_data,
                    "LISTEN" => theme.tcp_info_listen_data,
                    "TIME_WAIT" => theme.tcp_info_common_data,
                    _ => theme.interface_name,
                };
                summary_lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {:<12}: ", state),
                        Style::default().fg(theme.interface_index),
                    ),
                    Span::styled(format!("{}", count), Style::default().fg(color)),
                ]));
            }

            let summary = Paragraph::new(summary_lines).block(
                Block::bordered()
                    .border_type(BorderType::Plain)
                    .title(" INFO ")
                    .title_style(
                        Style::default()
                            .fg(theme.tx_bar)
                            .add_modifier(Modifier::BOLD),
                    ),
            );
            frame.render_widget(summary, tcp_split[0]);

            let tcp_rows: Vec<Row> = filtered_tcp
                .iter()
                .enumerate()
                .map(|(i, conn)| {
                    let is_selected = i == display_index;

                    let local_addr = format!("{}:{}", format_ip(&conn.local_ip), conn.local_port);
                    let remote_addr =
                        format!("{}:{}", format_ip(&conn.remote_ip), conn.remote_port);
                    let state = tcp_state_name(conn.state);

                    let hostname = hostname_cache
                        .get(&conn.remote_ip)
                        .map(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();

                    let state_style = if is_selected {
                        Style::default().fg(Color::Yellow)
                    } else {
                        match state {
                            "ESTABLISHED" => Style::default().fg(theme.state_data),
                            "LISTEN" => Style::default().fg(theme.state_data),
                            "TIME_WAIT" => Style::default().fg(theme.state_data),
                            "CLOSE_WAIT" => Style::default().fg(theme.state_data),
                            "SYN_SENT" | "SYN_RECV" => Style::default().fg(theme.state_data),
                            "FIN_WAIT1" | "FIN_WAIT2" => Style::default().fg(theme.tx_drops),
                            _ => Style::default().fg(theme.interface_index),
                        }
                    };

                    let queue_style = if is_selected {
                        Style::default().fg(theme.state_data)
                    } else if conn.tx_queue > 0 || conn.rx_queue > 0 {
                        Style::default()
                            .fg(theme.tx_drops)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.uid_data)
                    };

                    let text_color = if is_selected {
                        theme.state_data
                    } else {
                        theme.local_addr_data
                    };

                    let hostname_color = if is_selected {
                        theme.state_data
                    } else {
                        theme.hostname_data
                    };

                    Row::new(vec![
                        Cell::from(Span::styled(
                            format!(" {}", local_addr),
                            Style::default().fg(text_color),
                        )),
                        Cell::from(Span::styled(
                            format!(" {}", remote_addr),
                            Style::default().fg(theme.remote_addr_data),
                        )),
                        Cell::from(Span::styled(
                            format!(" {}", hostname),
                            Style::default().fg(hostname_color),
                        )),
                        Cell::from(Span::styled(format!(" {}", state), state_style)),
                        Cell::from(Span::styled(
                            format!(" {}:{}", conn.tx_queue, conn.rx_queue),
                            queue_style,
                        )),
                        Cell::from(Span::styled(
                            format!(" {}", conn.uid),
                            Style::default().fg(theme.uid_data),
                        )),
                        Cell::from(Span::styled(
                            format!(" {}", conn.inode),
                            Style::default().fg(if is_selected {
                                theme.interface_name
                            } else {
                                theme.inode_data
                            }),
                        )),
                    ])
                })
                .collect();

            let total_rows = tcp_rows.len();
            let visible_rows = (tcp_split[1].height as usize).saturating_sub(4);

            let mut scroll_offset = app.tcp_vertical_scroll;

            if *index < scroll_offset {
                scroll_offset = *index;
            } else if *index >= scroll_offset + visible_rows {
                scroll_offset = (*index + 1).saturating_sub(visible_rows);
            }

            app.tcp_vertical_scroll = scroll_offset;
            app.tcp_vertical_scroll_state =
                app.tcp_vertical_scroll_state.content_length(total_rows);

            let visible_tcp_rows: Vec<Row> = tcp_rows
                .into_iter()
                .skip(scroll_offset)
                .take(visible_rows)
                .collect();

            let title = if filter.is_empty() {
                format!("TCP FILTER: * (↑ ↓ Enter Esc) ",)
            } else {
                format!("TCP FILTER: {} (↑ ↓ Enter Esc) ", filter)
            };

            let tcp_table = Table::new(
                visible_tcp_rows,
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(25),
                    Constraint::Length(12),
                    Constraint::Length(10),
                    Constraint::Length(6),
                    Constraint::Fill(1),
                ],
            )
            .header(
                Row::new(vec![
                    Cell::from(" Local Address"),
                    Cell::from(" Remote Address"),
                    Cell::from(" Hostname"),
                    Cell::from(" State"),
                    Cell::from(" TX:RX"),
                    Cell::from(" UID"),
                    Cell::from(" Inode"),
                ])
                .style(
                    Style::default()
                        .fg(theme.tcp_info_connections)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .block(
                Block::bordered()
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(theme.rx_error))
                    .title(title)
                    .title_style(
                        Style::default()
                            .fg(Color::Yellow)
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
                    .style(Style::default().fg(theme.filter)),
                tcp_split[1].inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut app.tcp_vertical_scroll_state,
            );
        }

        _ => {
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

            let mut summary_lines = vec![
                Line::from("  Connections").style(Style::default().fg(theme.tcp_info_connections)),
                Line::from(vec![
                    Span::styled(
                        "  Total       : ",
                        Style::default().fg(theme.interface_index),
                    ),
                    Span::styled(
                        format!("{}", tcp_data.len()),
                        Style::default()
                            .fg(theme.interface_name)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
            ];

            let unique_ips: HashSet<[u8; 4]> = tcp_data
                .iter()
                .filter(|c| c.remote_ip != [0, 0, 0, 0] && c.remote_ip != [127, 0, 0, 1])
                .map(|c| c.remote_ip)
                .collect();

            let active = tcp_data
                .iter()
                .filter(|c| c.tx_queue > 0 || c.rx_queue > 0)
                .count();
            let local_only = tcp_data
                .iter()
                .filter(|c| c.remote_ip == [127, 0, 0, 1] || c.remote_ip == [0, 0, 0, 0])
                .count();

            summary_lines.push(Line::from(vec![
                Span::styled(
                    "  Active      : ",
                    Style::default().fg(theme.interface_index),
                ),
                Span::styled(
                    format!("{}", active),
                    Style::default().fg(theme.tcp_info_active_data),
                ),
            ]));

            summary_lines.push(Line::from(vec![
                Span::styled(
                    "  Unique IPs  : ",
                    Style::default().fg(theme.interface_index),
                ),
                Span::styled(
                    format!("{}", unique_ips.len()),
                    Style::default().fg(theme.tcp_info_unique_ip_data),
                ),
            ]));

            summary_lines.push(Line::from(vec![
                Span::styled(
                    "  Local/Ext   : ",
                    Style::default().fg(theme.tcp_info_localext),
                ),
                Span::styled(
                    format!("{}", local_only),
                    Style::default().fg(theme.tcp_info_localext_data),
                ),
                Span::raw("/"),
                Span::styled(
                    format!("{}", tcp_data.len() - local_only),
                    Style::default().fg(theme.tcp_info_localext_data),
                ),
            ]));

            summary_lines.push(Line::from(""));

            for (state, count) in state_counts.iter() {
                let color = match *state {
                    "ESTABLISHED" => theme.tcp_info_established_data,
                    "LISTEN" => theme.tcp_info_listen_data,
                    "TIME_WAIT" => theme.tcp_info_common_data,
                    _ => theme.interface_name,
                };
                summary_lines.push(Line::from(vec![
                    Span::styled(
                        format!("  {:<12}: ", state),
                        Style::default().fg(theme.interface_index),
                    ),
                    Span::styled(format!("{}", count), Style::default().fg(color)),
                ]));
            }

            let summary = Paragraph::new(summary_lines).block(
                Block::bordered()
                    .border_type(BorderType::Plain)
                    .title(" INFO ")
                    .title_style(
                        Style::default()
                            .fg(theme.tx_bar)
                            .add_modifier(Modifier::BOLD),
                    ),
            );
            frame.render_widget(summary, tcp_split[0]);

            let tcp_rows: Vec<Row> = tcp_data
                .iter()
                .map(|conn| {
                    let local_addr = format!("{}:{}", format_ip(&conn.local_ip), conn.local_port);
                    let remote_addr =
                        format!("{}:{}", format_ip(&conn.remote_ip), conn.remote_port);
                    let state = tcp_state_name(conn.state);

                    let hostname = app
                        .hostname_cache_arc
                        .lock()
                        .unwrap()
                        .get(&conn.remote_ip)
                        .unwrap_or(&String::new())
                        .to_string();

                    let state_style = {
                        match state {
                            "ESTABLISHED" => Style::default().fg(theme.state_data),
                            "LISTEN" => Style::default().fg(theme.state_data),
                            "TIME_WAIT" => Style::default().fg(theme.state_data),
                            "CLOSE_WAIT" => Style::default().fg(theme.state_data),
                            "SYN_SENT" | "SYN_RECV" => Style::default().fg(theme.state_data),
                            "FIN_WAIT1" | "FIN_WAIT2" => Style::default().fg(theme.tx_drops),
                            _ => Style::default().fg(theme.interface_index),
                        }
                    };
                    let queue_style = if conn.tx_queue > 0 || conn.rx_queue > 0 {
                        Style::default()
                            .fg(theme.tx_drops)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(theme.uid_data)
                    };

                    Row::new(vec![
                        Cell::from(Span::styled(
                            format!(" {}", local_addr),
                            Style::default().fg(theme.local_addr_data),
                        )),
                        Cell::from(Span::styled(
                            format!(" {}", remote_addr),
                            Style::default().fg(theme.remote_addr_data),
                        )),
                        Cell::from(Span::styled(
                            format!(" {}", hostname),
                            Style::default().fg(theme.hostname_data),
                        )),
                        Cell::from(Span::styled(format!(" {}", state), state_style)),
                        Cell::from(Span::styled(
                            format!(" {}:{}", conn.tx_queue, conn.rx_queue),
                            queue_style,
                        )),
                        Cell::from(Span::styled(
                            format!(" {}", conn.uid),
                            Style::default().fg(theme.uid_data),
                        )),
                        Cell::from(Span::styled(
                            format!(" {}", conn.inode),
                            Style::default().fg(theme.inode_data),
                        )),
                    ])
                })
                .collect();

            let total_rows = tcp_rows.len();
            app.tcp_vertical_scroll_state =
                app.tcp_vertical_scroll_state.content_length(total_rows);

            let visible_rows = (tcp_split[1].height as usize).saturating_sub(4);
            let mut scroll_offset = app.tcp_vertical_scroll;
            if scroll_offset > total_rows.saturating_sub(visible_rows) {
                scroll_offset = total_rows.saturating_sub(visible_rows);
            }

            let visible_tcp_rows: Vec<Row> = tcp_rows
                .into_iter()
                .skip(scroll_offset)
                .take(visible_rows)
                .collect();

            let tcp_table = Table::new(
                visible_tcp_rows,
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(25),
                    Constraint::Length(12),
                    Constraint::Length(10),
                    Constraint::Length(6),
                    Constraint::Fill(1),
                ],
            )
            .header(
                Row::new(vec![
                    Cell::from(" Local Address"),
                    Cell::from(" Remote Address"),
                    Cell::from(" Hostname"),
                    Cell::from(" State"),
                    Cell::from(" TX:RX"),
                    Cell::from(" UID"),
                    Cell::from(" Inode"),
                ])
                .style(
                    Style::default()
                        .fg(theme.filter)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .block(
                Block::bordered()
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(tcp_border))
                    .title(Line::from(vec![
                        Span::styled(
                            " [f] ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("TCP CONNECTIONS ({})", tcp_data.len()),
                            Style::default().fg(tcp_border).add_modifier(Modifier::BOLD),
                        ),
                    ]))
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
                    .style(Style::default().fg(theme.interface_name)),
                tcp_split[1].inner(Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut app.tcp_vertical_scroll_state,
            );
        }
    }

    if app.show_help {
        render_help_popup(frame);
    }
    if app.enter_tick_active {
        draw_tick_mode(frame, app);
    }
    if app.edit_rx_mode || app.edit_tx_mode {
        draw_speed_edit_popup(frame, app);
    }
}

fn render_help_popup(frame: &mut Frame) {
    let area = frame.area();

    let popup_area = {
        let vertical =
            Layout::vertical([Constraint::Percentage(80)]).flex(ratatui::layout::Flex::Center);
        let horizontal =
            Layout::horizontal([Constraint::Percentage(70)]).flex(ratatui::layout::Flex::Center);
        let [area] = vertical.areas(area);
        let [area] = horizontal.areas(area);
        area
    };

    frame.render_widget(Clear, popup_area);

    let help_text = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  GLOBAL",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    q         ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit application"),
        ]),
        Line::from(vec![
            Span::styled("    ?         ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle this help menu"),
        ]),
        Line::from(vec![
            Span::styled("    Tab       ", Style::default().fg(Color::Cyan)),
            Span::raw("Switch focus between Interfaces and TCP Table"),
        ]),
        Line::from(vec![
            Span::styled("    K         ", Style::default().fg(Color::Cyan)),
            Span::raw("Change tick rate (refresh interval)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  INTERFACES",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    ↑ /↓      ", Style::default().fg(Color::Cyan)),
            Span::raw("Navigate interface list"),
        ]),
        Line::from(vec![
            Span::styled("    Enter     ", Style::default().fg(Color::Cyan)),
            Span::raw("Select interface / Select 'All'"),
        ]),
        Line::from(vec![
            Span::styled("    f         ", Style::default().fg(Color::Cyan)),
            Span::raw("Filter interfaces by name"),
        ]),
        Line::from(vec![
            Span::styled("    R/T       ", Style::default().fg(Color::Cyan)),
            Span::raw("Edit RX/TX speed limits"),
        ]),
        Line::from(vec![
            Span::styled("    b/d       ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle byte units (KiB, MiB, GiB) / (KB, MB, GB)"),
        ]),
        Line::from(vec![
            Span::styled("    r         ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle raw bytes display"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  TCP CONNECTIONS",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    ↑ /↓      ", Style::default().fg(Color::Cyan)),
            Span::raw("Scroll through connections"),
        ]),
        Line::from(vec![
            Span::styled("    f         ", Style::default().fg(Color::Cyan)),
            Span::raw("Filter connections (all fields)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "  FILTER MODE",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("    Type      ", Style::default().fg(Color::Cyan)),
            Span::raw("Search across all connection fields"),
        ]),
        Line::from(vec![
            Span::styled("              ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "(IP, hostname, state, UID, inode)",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("    ↑ /↓      ", Style::default().fg(Color::Cyan)),
            Span::raw("Navigate through filtered results"),
        ]),
        Line::from(vec![
            Span::styled("    Enter     ", Style::default().fg(Color::Cyan)),
            Span::raw("Lock onto selected connection"),
        ]),
        Line::from(vec![
            Span::styled("              ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "(Press Enter again to unlock)",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("    Esc       ", Style::default().fg(Color::Cyan)),
            Span::raw("Exit filter mode"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  Press ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "?",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" or ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "Esc",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close", Style::default().fg(Color::DarkGray)),
        ]),
        Line::from(""),
    ];

    let help_paragraph = Paragraph::new(help_text)
        .block(
            Block::bordered()
                .border_type(BorderType::Plain)
                .title(vec![
                    Span::raw(" "),
                    Span::styled("⌨", Style::default().fg(Color::Yellow)),
                    Span::raw("  "),
                    Span::styled(
                        "KEYBOARD SHORTCUTS",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw("  "),
                ])
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Left);

    frame.render_widget(help_paragraph, popup_area);
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
