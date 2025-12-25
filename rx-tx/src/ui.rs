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
use ratatui::widgets::TableState;
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
            Span::styled(" Current: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                current_tick,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" New Rate: ", Style::default().fg(Color::DarkGray)),
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
                " Enter ",
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
                " Interface: {}",
                app.editing_interface.as_ref().unwrap_or(&"".to_string())
            ),
            Style::default().fg(Color::Cyan),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled(format!(" {}: ", field_name), Style::default().fg(color)),
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
            " Enter: Save | Esc: Cancel",
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

#[derive(Clone)]
struct CommonColor {
    heading: Color,
    data: Color,
    muted: Color,
    tick: Color,
}

#[derive(Clone)]
struct OverviewAreaColor {
    border: Color,
    key: Color,
    val: Color,
}
#[derive(Clone)]
struct RxGraphAreaColor {
    color: Color,
}
#[derive(Clone)]

struct TxGraphAreaColor {
    color: Color,
}
#[derive(Clone)]

struct InterfaceAreaColor {
    border: Color,
    filter_highlight_symbol: Color,
    name: Color,
    filter: Color,
    activity_symbol: Color,
}

#[derive(Clone)]

struct RxBarAreaColor {
    border: Color,
    name: Color,
    common_heading: Color,
    current_val: Color,
    peak_val: Color,
    average_val: Color,
    link_speed_highlight: Color,
    link_speed_heading: Color,
    link_speed_val: Color,
    tick_highlight: Color,
    tick_heading: Color,
    tick_val: Color,
}
#[derive(Clone)]
struct TxBarAreaColor {
    border: Color,
    name: Color,
    common_heading: Color,
    current_val: Color,
    peak_val: Color,
    average_val: Color,
    link_speed_highlight: Color,
    link_speed_heading: Color,
    link_speed_val: Color,
}

// Will need to think whether to have individual colors to each element.
#[derive(Clone)]

struct InfoAreaColor {
    heading: Color,
    key: Color,
    val: Color,
}
#[derive(Clone)]

struct RxAreaColor {
    heading: Color,
    key: Color,
    val: Color,
}
#[derive(Clone)]

struct TxAreaColor {
    heading: Color,
    key: Color,
    val: Color,
}
#[derive(Clone)]

struct TcpInfoAreaColor {
    heading: Color,
    key: Color,
    val: Color,
}
#[derive(Clone)]

struct TcpConnAreaColor {
    border: Color,
    filter_highlight_symbol: Color,
    heading: Color,
    local_addr_val: Color,
    remote_addr_val: Color,
    hostname_val: Color,
    txrx_val: Color,
    uid_val: Color,
    inode_val: Color,
}

#[derive(Clone)]
pub struct Theme {
    interface_area_color: InterfaceAreaColor,
    overview_area_color: OverviewAreaColor,
    rxgraph_area_color: RxGraphAreaColor,
    txgraph_area_color: TxGraphAreaColor,
    rxbar_area_color: RxBarAreaColor,
    txbar_area_color: TxBarAreaColor,
    info_area_color: InfoAreaColor,
    rx_area_color: RxAreaColor,
    tx_area_color: TxAreaColor,
    tcpinfo_area_color: TcpInfoAreaColor,
    tcpconn_area_color: TcpConnAreaColor,
}

pub static THEMES: [(&'static str, std::sync::LazyLock<Theme>); 3] = [
    ("gruvbox", std::sync::LazyLock::new(|| gruvbox())),
    ("light", std::sync::LazyLock::new(|| light())),
    ("dark", std::sync::LazyLock::new(|| dark())),
];

impl Default for Theme {
    fn default() -> Self {
        let common = CommonColor {
            heading: Color::DarkGray,
            data: Color::White,
            muted: Color::Rgb(190, 190, 190),
            tick: Color::DarkGray,
        };

        Self {
            interface_area_color: InterfaceAreaColor {
                border: Color::Red,
                filter_highlight_symbol: Color::Yellow,
                name: Color::Red,
                filter: Color::Red,
                activity_symbol: Color::Green,
            },

            overview_area_color: OverviewAreaColor {
                border: Color::Yellow,
                key: common.heading,
                val: common.data,
            },

            rxgraph_area_color: RxGraphAreaColor {
                color: Color::Green,
            },

            txgraph_area_color: TxGraphAreaColor { color: Color::Blue },

            rxbar_area_color: RxBarAreaColor {
                border: Color::Green,
                name: Color::Green,
                common_heading: common.heading,
                current_val: Color::Blue,
                peak_val: Color::Green,
                average_val: Color::Blue,
                link_speed_highlight: Color::LightCyan,
                link_speed_heading: common.heading,
                link_speed_val: Color::Green,
                tick_highlight: Color::LightRed,
                tick_heading: common.heading,
                tick_val: Color::Blue,
            },

            txbar_area_color: TxBarAreaColor {
                border: Color::Blue,
                name: Color::Blue,
                common_heading: common.heading,
                current_val: Color::Green,
                peak_val: Color::Blue,
                average_val: Color::Green,
                link_speed_highlight: Color::LightGreen,
                link_speed_heading: common.heading,
                link_speed_val: Color::Blue,
            },

            info_area_color: InfoAreaColor {
                heading: Color::Yellow,
                key: common.heading,
                val: common.data,
            },

            rx_area_color: RxAreaColor {
                heading: Color::Green,
                key: common.heading,
                val: common.data,
            },

            tx_area_color: TxAreaColor {
                heading: Color::Blue,
                key: common.heading,
                val: common.data,
            },

            tcpinfo_area_color: TcpInfoAreaColor {
                heading: Color::Green,
                key: common.heading,
                val: common.data,
            },

            tcpconn_area_color: TcpConnAreaColor {
                border: Color::Blue,
                filter_highlight_symbol: Color::Yellow,
                heading: common.heading,
                local_addr_val: Color::Green,
                remote_addr_val: Color::Blue,
                hostname_val: common.muted,
                txrx_val: common.muted,
                uid_val: common.muted,
                inode_val: common.muted,
            },
        }
    }
}

fn gruvbox() -> Theme {
    let common = CommonColor {
        heading: Color::Rgb(146, 131, 116),
        data: Color::Rgb(235, 219, 178),
        muted: Color::Rgb(168, 153, 132),
        tick: Color::Rgb(146, 131, 116),
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Rgb(204, 36, 29),
            filter_highlight_symbol: Color::Rgb(250, 189, 47),
            name: Color::Rgb(251, 73, 52),
            filter: Color::Rgb(204, 36, 29),
            activity_symbol: Color::Rgb(184, 187, 38),
        },

        overview_area_color: OverviewAreaColor {
            border: Color::Rgb(215, 153, 33),
            key: common.heading,
            val: common.data,
        },

        rxgraph_area_color: RxGraphAreaColor {
            color: Color::Rgb(184, 187, 38),
        },

        txgraph_area_color: TxGraphAreaColor {
            color: Color::Rgb(131, 165, 152),
        },

        rxbar_area_color: RxBarAreaColor {
            border: Color::Rgb(152, 151, 26),
            name: Color::Rgb(184, 187, 38),
            common_heading: common.heading,
            current_val: Color::Rgb(131, 165, 152),
            peak_val: Color::Rgb(184, 187, 38),
            average_val: Color::Rgb(69, 133, 136),
            link_speed_highlight: Color::Rgb(142, 192, 124),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(184, 187, 38),
            tick_highlight: Color::Rgb(251, 73, 52),
            tick_heading: common.heading,
            tick_val: Color::Rgb(131, 165, 152),
        },

        txbar_area_color: TxBarAreaColor {
            border: Color::Rgb(69, 133, 136),
            name: Color::Rgb(131, 165, 152),
            common_heading: common.heading,
            current_val: Color::Rgb(184, 187, 38),
            peak_val: Color::Rgb(131, 165, 152),
            average_val: Color::Rgb(152, 151, 26),
            link_speed_highlight: Color::Rgb(184, 187, 38),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(131, 165, 152),
        },

        info_area_color: InfoAreaColor {
            heading: Color::Rgb(250, 189, 47),
            key: common.heading,
            val: common.data,
        },

        rx_area_color: RxAreaColor {
            heading: Color::Rgb(184, 187, 38),
            key: common.heading,
            val: common.data,
        },

        tx_area_color: TxAreaColor {
            heading: Color::Rgb(131, 165, 152),
            key: common.heading,
            val: common.data,
        },

        tcpinfo_area_color: TcpInfoAreaColor {
            heading: Color::Rgb(184, 187, 38),
            key: common.heading,
            val: common.data,
        },

        tcpconn_area_color: TcpConnAreaColor {
            border: Color::Rgb(69, 133, 136),
            filter_highlight_symbol: Color::Rgb(250, 189, 47),
            heading: common.heading,
            local_addr_val: Color::Rgb(184, 187, 38),
            remote_addr_val: Color::Rgb(131, 165, 152),
            hostname_val: common.muted,
            txrx_val: common.muted,
            uid_val: common.muted,
            inode_val: common.muted,
        },
    }
}

fn dark() -> Theme {
    let common = CommonColor {
        heading: Color::DarkGray,
        data: Color::White,
        muted: Color::Rgb(190, 190, 190),
        tick: Color::DarkGray,
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Blue,
            filter_highlight_symbol: Color::Yellow,
            name: Color::Red,
            filter: Color::Red,
            activity_symbol: Color::Green,
        },

        overview_area_color: OverviewAreaColor {
            border: Color::Yellow,
            key: common.heading,
            val: common.data,
        },

        rxgraph_area_color: RxGraphAreaColor {
            color: Color::Green,
        },

        txgraph_area_color: TxGraphAreaColor { color: Color::Blue },

        rxbar_area_color: RxBarAreaColor {
            border: Color::Green,
            name: Color::Green,
            common_heading: common.heading,
            current_val: Color::Blue,
            peak_val: Color::Green,
            average_val: Color::Blue,
            link_speed_highlight: Color::LightCyan,
            link_speed_heading: common.heading,
            link_speed_val: Color::Green,
            tick_highlight: Color::LightRed,
            tick_heading: common.heading,
            tick_val: Color::Blue,
        },

        txbar_area_color: TxBarAreaColor {
            border: Color::Blue,
            name: Color::Blue,
            common_heading: common.heading,
            current_val: Color::Green,
            peak_val: Color::Blue,
            average_val: Color::Green,
            link_speed_highlight: Color::LightGreen,
            link_speed_heading: common.heading,
            link_speed_val: Color::Blue,
        },

        info_area_color: InfoAreaColor {
            heading: Color::Yellow,
            key: common.heading,
            val: common.data,
        },

        rx_area_color: RxAreaColor {
            heading: Color::Green,
            key: common.heading,
            val: common.data,
        },

        tx_area_color: TxAreaColor {
            heading: Color::Blue,
            key: common.heading,
            val: common.data,
        },

        tcpinfo_area_color: TcpInfoAreaColor {
            heading: Color::Green,
            key: common.heading,
            val: common.data,
        },

        tcpconn_area_color: TcpConnAreaColor {
            border: Color::Blue,
            filter_highlight_symbol: Color::Yellow,
            heading: common.heading,
            local_addr_val: Color::Green,
            remote_addr_val: Color::Blue,
            hostname_val: common.muted,
            txrx_val: common.muted,
            uid_val: common.muted,
            inode_val: common.muted,
        },
    }
}

fn light() -> Theme {
    let common = CommonColor {
        heading: Color::DarkGray,
        data: Color::White,
        muted: Color::Rgb(190, 190, 190),
        tick: Color::DarkGray,
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::White,
            filter_highlight_symbol: Color::Yellow,
            name: Color::Red,
            filter: Color::Red,
            activity_symbol: Color::Green,
        },

        overview_area_color: OverviewAreaColor {
            border: Color::Yellow,
            key: common.heading,
            val: common.data,
        },

        rxgraph_area_color: RxGraphAreaColor {
            color: Color::Green,
        },

        txgraph_area_color: TxGraphAreaColor { color: Color::Blue },

        rxbar_area_color: RxBarAreaColor {
            border: Color::Green,
            name: Color::Green,
            common_heading: common.heading,
            current_val: Color::Blue,
            peak_val: Color::Green,
            average_val: Color::Blue,
            link_speed_highlight: Color::LightCyan,
            link_speed_heading: common.heading,
            link_speed_val: Color::Green,
            tick_highlight: Color::LightRed,
            tick_heading: common.heading,
            tick_val: Color::Blue,
        },

        txbar_area_color: TxBarAreaColor {
            border: Color::Blue,
            name: Color::Blue,
            common_heading: common.heading,
            current_val: Color::Green,
            peak_val: Color::Blue,
            average_val: Color::Green,
            link_speed_highlight: Color::LightGreen,
            link_speed_heading: common.heading,
            link_speed_val: Color::Blue,
        },

        info_area_color: InfoAreaColor {
            heading: Color::Yellow,
            key: common.heading,
            val: common.data,
        },

        rx_area_color: RxAreaColor {
            heading: Color::Green,
            key: common.heading,
            val: common.data,
        },

        tx_area_color: TxAreaColor {
            heading: Color::Blue,
            key: common.heading,
            val: common.data,
        },

        tcpinfo_area_color: TcpInfoAreaColor {
            heading: Color::Green,
            key: common.heading,
            val: common.data,
        },

        tcpconn_area_color: TcpConnAreaColor {
            border: Color::Blue,
            filter_highlight_symbol: Color::Yellow,
            heading: common.heading,
            local_addr_val: Color::Green,
            remote_addr_val: Color::Blue,
            hostname_val: common.muted,
            txrx_val: common.muted,
            uid_val: common.muted,
            inode_val: common.muted,
        },
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
    let interface_border = if app.focus == Focus::Interfaces {
        app.current_theme
            .interface_area_color
            .filter_highlight_symbol
    } else {
        app.current_theme.interface_area_color.border
    };

    let tcp_border = if app.focus == Focus::TcpTable {
        app.current_theme.tcpconn_area_color.filter_highlight_symbol
    } else {
        app.current_theme.tcpconn_area_color.border
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
            let filtered: Vec<(usize, &String)> = interface_names
                .iter()
                .enumerate()
                .filter(|(_, name)| name.contains(filter))
                .collect();

            let items = interface_vec_items(
                Some(&filtered),
                &app.current_theme,
                &rx_data_strings,
                &tx_data_lines,
                &interface_names,
            );

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
                        .title_style(
                            Style::default()
                                // add this explicitly in the theme
                                .fg(app.current_theme.interface_area_color.filter)
                                .add_modifier(Modifier::BOLD),
                        ),
                )
                .highlight_style(
                    Style::default()
                        .bg(app.current_theme.interface_area_color.border)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_spacing(HighlightSpacing::Always)
                .fg(app.current_theme.interface_area_color.border);

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
            let items = interface_vec_items(
                None,
                &app.current_theme,
                &rx_data_strings,
                &tx_data_lines,
                &interface_names,
            );

            let interface_count = interface_names.iter().count();
            let mut state = ListState::default();
            state.select(Some(app.vertical_scroll));

            let title = Line::from(vec![
                Span::styled(
                    " [f]",
                    Style::default()
                        .fg(interface_border)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!(" Interfaces ({interface_count}) ")),
            ]);

            let list = List::new(items).block(
                Block::bordered()
                    .border_type(BorderType::Plain)
                    .title(title)
                    .fg(app.current_theme.interface_area_color.border),
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
                        Span::raw(" "),
                        Span::styled(
                            make_bar(rx_load, bar_width),
                            Style::default().fg(app.current_theme.rxbar_area_color.border),
                        ),
                        Span::raw(" ▼").fg(app.current_theme.rxbar_area_color.border),
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
                                    .fg(app.current_theme.rxbar_area_color.name)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled(
                                "Cur: ",
                                Style::default()
                                    .fg(app.current_theme.rxbar_area_color.common_heading),
                            ),
                            Span::styled(
                                format!("{:<10}", rx_speed_str),
                                Style::default()
                                    .fg(app.current_theme.rxbar_area_color.current_val)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled(
                                "Peak: ",
                                Style::default()
                                    .fg(app.current_theme.rxbar_area_color.common_heading),
                            ),
                            Span::styled(
                                format!("{:<10}", rx_peak_str),
                                Style::default()
                                    .fg(app.current_theme.rxbar_area_color.peak_val)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled(
                                "Avg: ",
                                Style::default()
                                    .fg(app.current_theme.rxbar_area_color.common_heading),
                            ),
                            Span::styled(
                                rx_avg_str,
                                Style::default()
                                    .fg(app.current_theme.rxbar_area_color.average_val)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" "),
                        ])
                        .title_top(
                            Line::from(vec![
                                Span::styled(
                                    " [R]",
                                    Style::default().fg(app
                                        .current_theme
                                        .rxbar_area_color
                                        .link_speed_highlight),
                                ),
                                Span::styled(
                                    " Link Speed: ",
                                    Style::default()
                                        .fg(app.current_theme.rxbar_area_color.link_speed_heading),
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
                                        .fg(app.current_theme.rxbar_area_color.link_speed_val)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(" │ "),
                                Span::styled(
                                    "[K] ",
                                    Style::default()
                                        .fg(app.current_theme.rxbar_area_color.tick_highlight),
                                ),
                                Span::styled(
                                    "Tick: ",
                                    Style::default()
                                        .fg(app.current_theme.rxbar_area_color.tick_heading),
                                ),
                                Span::styled(
                                    tick_display.clone(),
                                    Style::default()
                                        .fg(app.current_theme.rxbar_area_color.tick_val)
                                        .add_modifier(Modifier::BOLD),
                                ),
                                Span::raw(" "),
                            ])
                            .right_aligned(),
                        ),
                )
                .fg(app.current_theme.rxbar_area_color.border)
                .alignment(Alignment::Left);

                frame.render_widget(rx_para, detail_chunks[0]);

                let tx_para = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(vec![
                        Span::raw(" "),
                        Span::styled(
                            make_bar(tx_load, bar_width),
                            Style::default().fg(app.current_theme.txbar_area_color.border),
                        ),
                        Span::raw(" ▲").fg(app.current_theme.txbar_area_color.border),
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
                                    .fg(app.current_theme.txbar_area_color.name)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled(
                                "Cur: ",
                                Style::default()
                                    .fg(app.current_theme.txbar_area_color.common_heading),
                            ),
                            Span::styled(
                                format!("{:<10}", tx_speed_str),
                                Style::default()
                                    .fg(app.current_theme.txbar_area_color.current_val)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled(
                                "Peak: ",
                                Style::default()
                                    .fg(app.current_theme.txbar_area_color.common_heading),
                            ),
                            Span::styled(
                                format!("{:<10}", tx_peak_str),
                                Style::default()
                                    .fg(app.current_theme.txbar_area_color.peak_val)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" │ "),
                            Span::styled(
                                "Avg: ",
                                Style::default()
                                    .fg(app.current_theme.txbar_area_color.common_heading),
                            ),
                            Span::styled(
                                tx_avg_str,
                                Style::default()
                                    .fg(app.current_theme.txbar_area_color.average_val)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" "),
                        ])
                        .border_style(
                            Style::default().fg(app.current_theme.txbar_area_color.border),
                        )
                        .title_top(
                            Line::from(vec![
                                Span::styled(
                                    " [T]",
                                    Style::default().fg(app
                                        .current_theme
                                        .txbar_area_color
                                        .link_speed_highlight),
                                )
                                .add_modifier(Modifier::BOLD),
                                Span::styled(
                                    " Link Speed: ",
                                    Style::default()
                                        .fg(app.current_theme.txbar_area_color.link_speed_heading),
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
                                        .fg(app.current_theme.txbar_area_color.link_speed_val)
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
                        Span::styled(
                            " Name        : ",
                            Style::default().fg(app.current_theme.info_area_color.key),
                        ),
                        Span::styled(
                            &interface_data.name,
                            Style::default()
                                .fg(app.current_theme.info_area_color.val)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " Total       : ",
                            Style::default().fg(app.current_theme.info_area_color.key),
                        ),
                        Span::styled(
                            total_str,
                            Style::default().fg(app.current_theme.info_area_color.val),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            " RX Bytes    : ",
                            Style::default().fg(app.current_theme.info_area_color.key),
                        ),
                        Span::styled(
                            rx_bytes_str,
                            Style::default().fg(app.current_theme.info_area_color.val),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " RX Packets  : ",
                            Style::default().fg(app.current_theme.info_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.receive.packets),
                            Style::default().fg(app.current_theme.info_area_color.val),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            " TX Bytes    : ",
                            Style::default().fg(app.current_theme.info_area_color.key),
                        ),
                        Span::styled(
                            tx_bytes_str,
                            Style::default().fg(app.current_theme.info_area_color.val),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " TX Packets  : ",
                            Style::default().fg(app.current_theme.info_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.transmit.packets),
                            Style::default().fg(app.current_theme.info_area_color.val),
                        ),
                    ]),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Plain)
                        .title(" INFO ")
                        .title_style(Style::new().bold())
                        .border_style(
                            Style::default().fg(app.current_theme.info_area_color.heading),
                        ),
                );

                let middle_col = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled(
                            " Errors      : ",
                            Style::default().fg(app.current_theme.rx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.receive.errs),
                            Style::default().fg(app.current_theme.rx_area_color.val),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " Drops       : ",
                            Style::default().fg(app.current_theme.rx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.receive.drop),
                            Style::default().fg(app.current_theme.rx_area_color.val),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            " FIFO        : ",
                            Style::default().fg(app.current_theme.rx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.receive.fifo),
                            Style::default().fg(app.current_theme.rx_area_color.val),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " Frame       : ",
                            Style::default().fg(app.current_theme.rx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.receive.frame),
                            Style::default().fg(app.current_theme.rx_area_color.val),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            " Compressed  : ",
                            Style::default().fg(app.current_theme.rx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.receive.compressed),
                            Style::default().fg(app.current_theme.rx_area_color.val),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " Multicast   : ",
                            Style::default().fg(app.current_theme.rx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.receive.multicast),
                            Style::default().fg(app.current_theme.rx_area_color.val),
                        ),
                    ]),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Plain)
                        .title(" RX ")
                        .title_style(Style::new().bold())
                        .border_style(Style::default().fg(app.current_theme.rx_area_color.heading)),
                );

                let right_col = Paragraph::new(vec![
                    Line::from(vec![
                        Span::styled(
                            " Errors       : ",
                            Style::default().fg(app.current_theme.tx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.transmit.errs),
                            Style::default().fg(app.current_theme.tx_area_color.val),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " Drops        : ",
                            Style::default().fg(app.current_theme.tx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.transmit.drop),
                            Style::default().fg(app.current_theme.tx_area_color.val),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            " FIFO         : ",
                            Style::default().fg(app.current_theme.tx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.transmit.fifo),
                            Style::default().fg(app.current_theme.tx_area_color.val),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " Collisions   : ",
                            Style::default().fg(app.current_theme.tx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.transmit.colls),
                            Style::default().fg(app.current_theme.tx_area_color.val),
                        ),
                    ]),
                    Line::from(""),
                    Line::from(vec![
                        Span::styled(
                            " Carrier      : ",
                            Style::default().fg(app.current_theme.tx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.transmit.carrier),
                            Style::default().fg(app.current_theme.tx_area_color.val),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled(
                            " Compressed   : ",
                            Style::default().fg(app.current_theme.tx_area_color.key),
                        ),
                        Span::styled(
                            format!("{}", interface_data.transmit.compressed),
                            Style::default().fg(app.current_theme.tx_area_color.val),
                        ),
                    ]),
                ])
                .block(
                    Block::bordered()
                        .border_type(BorderType::Plain)
                        .title(" TX ")
                        .title_style(Style::new().bold())
                        .border_style(Style::default().fg(app.current_theme.tx_area_color.heading)),
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
                        " System Uptime       : ",
                        Style::default().fg(app.current_theme.overview_area_color.key),
                    ),
                    Span::styled(
                        uptime,
                        Style::default().fg(app.current_theme.overview_area_color.val),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        " Total RX            : ",
                        Style::default().fg(app.current_theme.overview_area_color.key),
                    ),
                    Span::styled(
                        summary_rx_val,
                        Style::default().fg(app.current_theme.overview_area_color.val),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        " Total TX            : ",
                        Style::default().fg(app.current_theme.overview_area_color.key),
                    ),
                    Span::styled(
                        summary_tx_val,
                        Style::default().fg(app.current_theme.overview_area_color.val),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        " Total Packets       : ",
                        Style::default().fg(app.current_theme.overview_area_color.key),
                    ),
                    Span::styled(
                        format!("{}", totals.total_packets),
                        Style::default().fg(app.current_theme.overview_area_color.val),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        " Total Errors        : ",
                        Style::default().fg(app.current_theme.overview_area_color.key),
                    ),
                    Span::styled(
                        format!("{}", totals.total_errors),
                        if totals.total_errors > 0 {
                            Style::default()
                                .fg(app.current_theme.overview_area_color.val)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(app.current_theme.overview_area_color.val)
                        },
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        " Total Drops         : ",
                        Style::default().fg(app.current_theme.overview_area_color.key),
                    ),
                    Span::styled(
                        format!("{}", totals.total_drops),
                        Style::default().fg(app.current_theme.overview_area_color.val),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        " Error Rate Ratio    : ",
                        Style::default().fg(app.current_theme.overview_area_color.key),
                    ),
                    Span::styled(
                        if totals.error_rate_pct > 0.0 {
                            format!("{:.3}%", totals.error_rate_pct)
                        } else {
                            "-".to_string()
                        },
                        Style::default().fg(app.current_theme.overview_area_color.val),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        " Drop Rate Ratio     : ",
                        Style::default().fg(app.current_theme.overview_area_color.key),
                    ),
                    Span::styled(
                        if totals.drop_rate_pct > 0.0 {
                            format!("{:.3}%", totals.drop_rate_pct)
                        } else {
                            "-".to_string()
                        },
                        Style::default().fg(app.current_theme.overview_area_color.val),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        " RX/TX Bytes Ratio   : ",
                        Style::default().fg(app.current_theme.overview_area_color.key),
                    ),
                    Span::styled(
                        if totals.rx_tx_bytes_ratio > 0.0 {
                            format!("{:.2}", totals.rx_tx_bytes_ratio)
                        } else {
                            "-".to_string()
                        },
                        Style::default().fg(app.current_theme.overview_area_color.val),
                    ),
                ]),
                Line::from(vec![
                    Span::styled(
                        " RX/TX Packets Ratio : ",
                        Style::default().fg(app.current_theme.overview_area_color.key),
                    ),
                    Span::styled(
                        if totals.rx_tx_packets_ratio > 0.0 {
                            format!("{:.2}", totals.rx_tx_packets_ratio)
                        } else {
                            "-".to_string()
                        },
                        Style::default().fg(app.current_theme.overview_area_color.val),
                    ),
                ]),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(""),
                Line::from(vec![Span::styled(
                    " Press `h` or `?` for help",
                    Style::default().fg(app.current_theme.overview_area_color.val),
                )]),
            ])
            .block(
                Block::bordered()
                    .border_type(BorderType::Plain)
                    .title_top(Line::from(" OVERVIEW ").left_aligned())
                    .title_top(
                        Line::from(vec![
                            // Change this
                            Span::styled(" [K]", Style::default().fg(Color::Red)),
                            Span::styled(" Tick: ", Style::default().fg(Color::DarkGray)),
                            Span::styled(
                                tick_display,
                                Style::default()
                                    .fg(app.current_theme.overview_area_color.border)
                                    .add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(" "),
                        ])
                        .right_aligned(),
                    )
                    .border_style(Style::new().fg(Color::Yellow)),
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
            let mut summary_lines = vec![Line::from(vec![
                Span::styled(
                    " Total       : ",
                    Style::default().fg(app.current_theme.tcpinfo_area_color.key),
                ),
                Span::styled(
                    format!("{}", tcp_data.len()),
                    Style::default()
                        .fg(app.current_theme.tcpinfo_area_color.val)
                        .add_modifier(Modifier::BOLD),
                ),
            ])];

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
                    " Active      : ",
                    Style::default().fg(app.current_theme.tcpinfo_area_color.key),
                ),
                Span::styled(
                    format!("{}", active),
                    Style::default().fg(app.current_theme.tcpinfo_area_color.val),
                ),
            ]));

            summary_lines.push(Line::from(vec![
                Span::styled(
                    " Unique IPs  : ",
                    Style::default().fg(app.current_theme.tcpinfo_area_color.key),
                ),
                Span::styled(
                    format!("{}", unique_ips.len()),
                    Style::default().fg(app.current_theme.tcpinfo_area_color.val),
                ),
            ]));

            summary_lines.push(Line::from(vec![
                Span::styled(
                    " Local/Ext   : ",
                    Style::default().fg(app.current_theme.tcpinfo_area_color.key),
                ),
                Span::styled(
                    format!("{}", local_only),
                    Style::default().fg(app.current_theme.tcpinfo_area_color.val),
                ),
                Span::raw("/"),
                Span::styled(
                    format!("{}", tcp_data.len() - local_only),
                    Style::default().fg(app.current_theme.tcpinfo_area_color.val),
                ),
            ]));

            summary_lines.push(Line::from(""));
            for (state, count) in state_counts.iter() {
                //Change this
                let color = match *state {
                    "ESTABLISHED" => Color::Green,
                    "LISTEN" => Color::Yellow,
                    "TIME_WAIT" => Color::Rgb(190, 190, 190),
                    _ => Color::Magenta,
                };
                summary_lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {:<12}: ", state),
                        Style::default().fg(app.current_theme.tcpinfo_area_color.key),
                    ),
                    Span::styled(format!("{}", count), Style::default().fg(color)),
                ]));
            }

            let summary = Paragraph::new(summary_lines)
                .block(
                    Block::bordered()
                        .border_type(BorderType::Plain)
                        .title(" INFO ")
                        .add_modifier(Modifier::BOLD),
                )
                .fg(Color::Green);
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
                        Style::default()
                            .fg(Color::White)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        match state {
                            "ESTABLISHED" => Style::default().fg(Color::Green),
                            "LISTEN" => Style::default().fg(Color::Yellow),
                            "TIME_WAIT" => Style::default().fg(Color::DarkGray),
                            "CLOSE_WAIT" => Style::default().fg(Color::Rgb(200, 140, 60)),
                            "SYN_SENT" | "SYN_RECV" => Style::default()
                                .fg(Color::Rgb(220, 180, 60))
                                .add_modifier(Modifier::BOLD),
                            "FIN_WAIT1" | "FIN_WAIT2" => {
                                Style::default().fg(Color::Rgb(180, 100, 140))
                            }
                            _ => Style::default().fg(Color::Gray),
                        }
                    };

                    let queue_style = if is_selected {
                        Style::default().fg(Color::White)
                    } else if conn.tx_queue > 0 || conn.rx_queue > 0 {
                        Style::default()
                            .fg(app.current_theme.tcpconn_area_color.txrx_val)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(app.current_theme.tcpconn_area_color.uid_val)
                    };

                    let (
                        local_addr_color,
                        remote_addr_color,
                        hostname_color,
                        tx_rx_color,
                        uid_color,
                        inode_color,
                    ) = if is_selected {
                        (
                            Color::White,
                            Color::White,
                            Color::White,
                            Color::White,
                            Color::White,
                            Color::White,
                        )
                    } else {
                        (
                            app.current_theme.tcpconn_area_color.local_addr_val,
                            app.current_theme.tcpconn_area_color.remote_addr_val,
                            app.current_theme.tcpconn_area_color.hostname_val,
                            app.current_theme.tcpconn_area_color.txrx_val,
                            app.current_theme.tcpconn_area_color.uid_val,
                            app.current_theme.tcpconn_area_color.inode_val,
                        )
                    };

                    Row::new(vec![
                        Cell::from(Span::styled(
                            format!("{}", local_addr),
                            Style::default().fg(local_addr_color),
                        )),
                        Cell::from(Span::styled(
                            format!("{}", remote_addr),
                            Style::default().fg(remote_addr_color),
                        )),
                        Cell::from(Span::styled(
                            format!("{}", hostname),
                            Style::default().fg(hostname_color),
                        )),
                        Cell::from(Span::styled(format!("{}", state), state_style)),
                        Cell::from(Span::styled(
                            format!("{}:{}", conn.tx_queue, conn.rx_queue),
                            queue_style,
                        )),
                        Cell::from(Span::styled(
                            format!("{}", conn.uid),
                            Style::default().fg(uid_color),
                        )),
                        Cell::from(Span::styled(
                            format!("{}", conn.inode),
                            Style::default().fg(inode_color),
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
                format!(" Filter: * (↑ ↓ Enter Esc) ",)
            } else {
                format!(" Filter: {} (↑ ↓ Enter Esc) ", filter)
            };

            let tcp_table = Table::new(
                visible_tcp_rows.clone(),
                [
                    Constraint::Percentage(20),
                    Constraint::Percentage(20),
                    Constraint::Percentage(30),
                    Constraint::Length(12),
                    Constraint::Length(10),
                    Constraint::Length(6),
                    Constraint::Fill(1),
                ],
            )
            .header(
                Row::new(vec![
                    Cell::from("Local Address"),
                    Cell::from("Remote Address"),
                    Cell::from("Hostname"),
                    Cell::from("State"),
                    Cell::from("TX:RX"),
                    Cell::from("UID"),
                    Cell::from("Inode"),
                ])
                .style(Style::default().fg(Color::DarkGray)),
            )
            .block(
                Block::bordered()
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(Color::Blue))
                    .title(title)
                    .title_style(
                        Style::default()
                            .fg(Color::Blue)
                            .add_modifier(Modifier::BOLD),
                    )
                    .padding(ratatui::widgets::Padding {
                        left: 1,
                        right: 2,
                        top: 0,
                        bottom: 0,
                    }),
            )
            .row_highlight_style(Style::default().bg(Color::Red).add_modifier(Modifier::BOLD));

            let mut state = TableState::new();
            let sel = (*index).min(visible_tcp_rows.len());
            state.select(Some(sel));

            frame.render_stateful_widget(tcp_table, tcp_split[1], &mut state);

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .begin_symbol(None)
                    .end_symbol(None)
                    .track_symbol(None)
                    .thumb_symbol("┃"),
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

            let mut summary_lines = vec![Line::from(vec![
                Span::styled(
                    " Total       : ",
                    Style::default().fg(app.current_theme.tcpinfo_area_color.key),
                ),
                Span::styled(
                    format!("{}", tcp_data.len()),
                    Style::default()
                        .fg(app.current_theme.tcpinfo_area_color.val)
                        .add_modifier(Modifier::BOLD),
                ),
            ])];

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
                    " Active      : ",
                    Style::default().fg(app.current_theme.tcpinfo_area_color.key),
                ),
                Span::styled(
                    format!("{}", active),
                    Style::default().fg(app.current_theme.tcpinfo_area_color.val),
                ),
            ]));

            summary_lines.push(Line::from(vec![
                Span::styled(
                    " Unique IPs  : ",
                    Style::default().fg(app.current_theme.tcpinfo_area_color.key),
                ),
                Span::styled(
                    format!("{}", unique_ips.len()),
                    Style::default().fg(app.current_theme.tcpinfo_area_color.val),
                ),
            ]));

            summary_lines.push(Line::from(vec![
                Span::styled(
                    " Local/Ext   : ",
                    Style::default().fg(app.current_theme.tcpinfo_area_color.key),
                ),
                Span::styled(
                    format!("{}/", local_only),
                    Style::default().fg(app.current_theme.tcpinfo_area_color.val),
                ),
                Span::styled(
                    format!("{}", tcp_data.len() - local_only),
                    Style::default().fg(app.current_theme.tcpinfo_area_color.val),
                ),
            ]));

            summary_lines.push(Line::from(""));

            for (state, count) in state_counts.iter() {
                //Change this
                let color = match *state {
                    "ESTABLISHED" => Color::Green,
                    "LISTEN" => Color::Yellow,
                    "TIME_WAIT" => Color::Rgb(190, 190, 190),
                    _ => Color::Magenta,
                };
                summary_lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {:<12}: ", state),
                        Style::default().fg(app.current_theme.tcpinfo_area_color.key),
                    ),
                    Span::styled(format!("{}", count), Style::default().fg(color)),
                ]));
            }

            let summary = Paragraph::new(summary_lines)
                .block(
                    Block::bordered()
                        .border_type(BorderType::Plain)
                        .title(" INFO ")
                        .title_style(Style::default().add_modifier(Modifier::BOLD)),
                )
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD);
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
                            "ESTABLISHED" => Style::default().fg(Color::Green),
                            "LISTEN" => Style::default().fg(Color::Yellow),
                            "TIME_WAIT" => Style::default().fg(Color::DarkGray),
                            "CLOSE_WAIT" => Style::default().fg(Color::Rgb(200, 140, 60)),
                            "SYN_SENT" | "SYN_RECV" => Style::default()
                                .fg(Color::Rgb(220, 180, 60))
                                .add_modifier(Modifier::BOLD),
                            "FIN_WAIT1" | "FIN_WAIT2" => {
                                Style::default().fg(Color::Rgb(180, 100, 140))
                            }
                            _ => Style::default().fg(Color::Rgb(190, 190, 190)),
                        }
                    };

                    let (
                        local_addr_color,
                        remote_addr_color,
                        hostname_color,
                        tx_rx_color,
                        uid_color,
                        inode_color,
                    ) = (
                        app.current_theme.tcpconn_area_color.local_addr_val,
                        app.current_theme.tcpconn_area_color.remote_addr_val,
                        app.current_theme.tcpconn_area_color.hostname_val,
                        app.current_theme.tcpconn_area_color.txrx_val,
                        app.current_theme.tcpconn_area_color.uid_val,
                        app.current_theme.tcpconn_area_color.inode_val,
                    );

                    Row::new(vec![
                        Cell::from(Span::styled(
                            format!("{}", local_addr),
                            Style::default().fg(local_addr_color),
                        )),
                        Cell::from(Span::styled(
                            format!("{}", remote_addr),
                            Style::default().fg(remote_addr_color),
                        )),
                        Cell::from(Span::styled(
                            format!("{}", hostname),
                            Style::default().fg(hostname_color),
                        )),
                        Cell::from(Span::styled(format!("{}", state), state_style)),
                        Cell::from(Span::styled(
                            format!("{}:{}", conn.tx_queue, conn.rx_queue),
                            Color::Rgb(190, 190, 190),
                        )),
                        Cell::from(Span::styled(
                            format!("{}", conn.uid),
                            Style::default().fg(uid_color),
                        )),
                        Cell::from(Span::styled(
                            format!("{}", conn.inode),
                            Style::default().fg(inode_color),
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
                    Constraint::Percentage(30),
                    Constraint::Length(12),
                    Constraint::Length(10),
                    Constraint::Length(6),
                    Constraint::Fill(1),
                ],
            )
            .header(
                Row::new(vec![
                    Cell::from("Local Address"),
                    Cell::from("Remote Address"),
                    Cell::from("Hostname"),
                    Cell::from("State"),
                    Cell::from("TX:RX"),
                    Cell::from("UID"),
                    Cell::from("Inode"),
                ])
                .style(
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD),
                ),
            )
            .block(
                Block::bordered()
                    .border_type(BorderType::Plain)
                    .border_style(Style::default().fg(Color::Blue))
                    .title(Line::from(vec![
                        Span::styled(
                            " [f] ",
                            Style::default().fg(tcp_border).add_modifier(Modifier::BOLD),
                        ),
                        Span::from(format!("Tcp Connections ({}) ", tcp_data.len())),
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
                    .thumb_symbol("┃"),
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
    if app.change_theme {
        theme_selection_popup(frame, app);
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
            " GLOBAL",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("   q         ", Style::default().fg(Color::Cyan)),
            Span::raw("Quit application"),
        ]),
        Line::from(vec![
            Span::styled("   ?         ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle this help menu"),
        ]),
        Line::from(vec![
            Span::styled("   Tab       ", Style::default().fg(Color::Cyan)),
            Span::raw("Switch focus between Interfaces and TCP Table"),
        ]),
        Line::from(vec![
            Span::styled("   K         ", Style::default().fg(Color::Cyan)),
            Span::raw("Change tick rate (refresh interval)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " INTERFACES",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("   ↑ /↓      ", Style::default().fg(Color::Cyan)),
            Span::raw("Navigate interface list"),
        ]),
        Line::from(vec![
            Span::styled("   Enter     ", Style::default().fg(Color::Cyan)),
            Span::raw("Select interface / Select 'All'"),
        ]),
        Line::from(vec![
            Span::styled("   f         ", Style::default().fg(Color::Cyan)),
            Span::raw("Filter interfaces by name"),
        ]),
        Line::from(vec![
            Span::styled("   R/T       ", Style::default().fg(Color::Cyan)),
            Span::raw("Edit RX/TX speed limits"),
        ]),
        Line::from(vec![
            Span::styled("   b/d       ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle byte units (KiB, MiB, GiB) / (KB, MB, GB)"),
        ]),
        Line::from(vec![
            Span::styled("   r         ", Style::default().fg(Color::Cyan)),
            Span::raw("Toggle raw bytes display"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " TCP CONNECTIONS",
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("   ↑ /↓      ", Style::default().fg(Color::Cyan)),
            Span::raw("Scroll through connections"),
        ]),
        Line::from(vec![
            Span::styled("   f         ", Style::default().fg(Color::Cyan)),
            Span::raw("Filter connections (all fields)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            " FILTER MODE",
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("   Type      ", Style::default().fg(Color::Cyan)),
            Span::raw("Search across all connection fields"),
        ]),
        Line::from(vec![
            Span::styled("             ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "(IP, hostname, state, UID, inode)",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("   ↑ /↓      ", Style::default().fg(Color::Cyan)),
            Span::raw("Navigate through filtered results"),
        ]),
        Line::from(vec![
            Span::styled("   Enter     ", Style::default().fg(Color::Cyan)),
            Span::raw("Lock onto selected connection"),
        ]),
        Line::from(vec![
            Span::styled("             ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "(Press Enter again to unlock)",
                Style::default().fg(Color::DarkGray),
            ),
        ]),
        Line::from(vec![
            Span::styled("   Esc       ", Style::default().fg(Color::Cyan)),
            Span::raw("Exit filter mode"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(" Press ", Style::default().fg(Color::DarkGray)),
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
                    Span::raw(" "),
                    Span::styled(
                        "KEYBOARD SHORTCUTS",
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(" "),
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

fn interface_vec_items<'a>(
    filtered_interfaces: Option<&Vec<(usize, &String)>>,
    theme: &Theme,
    rx_data_strings: &Vec<Line>,
    tx_data_lines: &Vec<Line>,
    interface_names: &Vec<String>,
) -> Vec<ListItem<'a>> {
    if let Some(filtered) = filtered_interfaces {
        filtered
            .iter()
            .enumerate()
            .map(|(display_idx, (original_idx, name))| {
                ListItem::new(vec![Line::from(vec![
                    Span::raw(" "),
                    Span::styled(format!("{:<16}", name), Style::default().fg(Color::White)),
                ])])
            })
            .collect()
    } else {
        interface_names
            .iter()
            .enumerate()
            .map(|(idx, name)| {
                ListItem::new(vec![Line::from(vec![
                    Span::raw(" "),
                    Span::styled(format!("{:<16}", name), Style::default().fg(Color::White)),
                ])])
            })
            .collect()
    }
}

fn theme_selection_popup(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let popup_area = Layout::vertical([
        Constraint::Percentage(40),
        Constraint::Percentage(40),
        Constraint::Percentage(40),
    ])
    .split(area)[1];

    let popup_area = Layout::horizontal([
        Constraint::Percentage(45),
        Constraint::Percentage(50),
        Constraint::Percentage(25),
    ])
    .split(popup_area)[1];

    let (title, color) = ("Themes", Color::Magenta);

    match &app.mode {
        Mode::SelectingTheme { filter, index } => {
            let chunks =
                Layout::vertical([Constraint::Length(3), Constraint::Min(1)]).split(popup_area);

            let filter_block = Paragraph::new(filter.as_str())
                .block(
                    Block::bordered()
                        .title("Filter")
                        .border_style(Style::default().fg(Color::Yellow)),
                )
                .style(Style::default().fg(Color::White));

            frame.render_widget(Clear, popup_area);
            frame.render_widget(filter_block, chunks[0]);

            let filtered: Vec<(usize, &str)> = THEMES
                .iter()
                .enumerate()
                .filter(|(_, (name, _))| name.contains(filter))
                .map(|(i, (name, _))| (i, *name))
                .collect();

            let list_items: Vec<ListItem> = filtered
                .iter()
                .map(|(_, name)| {
                    ListItem::new(Line::from(vec![
                        Span::raw(" "),
                        Span::styled(format!("{:<15}", name), Style::default().fg(Color::White)),
                    ]))
                })
                .collect();

            let mut state = ListState::default();
            let max_index = filtered.len().saturating_sub(1);
            let sel = (*index).min(max_index);
            state.select(Some(sel));

            let list = List::new(list_items)
                .block(
                    Block::bordered()
                        .border_style(Style::default().fg(color))
                        .title(title)
                        .title_style(Style::default().fg(color).add_modifier(Modifier::BOLD)),
                )
                .highlight_style(Style::default().bg(Color::Red).add_modifier(Modifier::BOLD))
                .highlight_symbol("> ")
                .highlight_spacing(HighlightSpacing::Always);

            frame.render_stateful_widget(list, chunks[1], &mut state);
        }
        _ => {}
    }
}
