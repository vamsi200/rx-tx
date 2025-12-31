use crate::app::App;
use crate::models::{self, *};
use crate::ui::{Theme, THEMES};
use anyhow::{anyhow, Error, Ok, Result};
use clap::builder::Str;
use core::net;
use crossterm::event::{self, Event};
use ratatui::crossterm::event::read;
use ratatui::style::{Modifier, Style};
use ratatui::symbols::line::{self, NORMAL};
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::Terminal;
use ratatui::{text::Text, Frame};
use std::collections::{HashMap, HashSet};
use std::fmt::format;
use std::fs::{self, rename, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::ptr::copy_nonoverlapping;
use std::thread::{self, sleep};
use std::time::{Duration, Instant};
use std::vec;

pub fn parse_proc_net_dev() -> Result<Vec<NetworkStats>> {
    let mut output = Vec::new();
    let data_path = PathBuf::from("/proc/net/dev");

    if let Err(e) = fs::metadata(&data_path) {
        eprint!("{e}");
    } else {
        let file = File::open(data_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines().skip(2) {
            let line = line?;
            let s: Vec<&str> = line.split(":").collect();

            if s.len() < 2 {
                continue;
            }

            let interface = s[0].trim().to_string();
            let values: Vec<u64> = s[1]
                .trim()
                .split_whitespace()
                .filter_map(|x| x.parse().ok())
                .collect();

            if values.len() != 16 {
                continue;
            }

            let receive = Receive {
                bytes: values[0],
                packets: values[1],
                errs: values[2],
                drop: values[3],
                fifo: values[4],
                frame: values[5],
                compressed: values[6],
                multicast: values[7],
            };
            let transmit = Transmit {
                bytes: values[8],
                packets: values[9],
                errs: values[10],
                drop: values[11],
                fifo: values[12],
                colls: values[13],
                carrier: values[14],
                compressed: values[15],
            };
            output.push(NetworkStats {
                name: interface,
                receive,
                transmit,
            });
        }
    }
    Ok(output)
}

pub fn get_network_interfaces(stats: &Vec<NetworkStats>) -> Vec<Line<'_>> {
    let lines: Vec<Line> = stats
        .iter()
        .flat_map(|interface| {
            let mut lines = vec![];

            lines.push(Line::from(format!(" {}", interface.name)));

            lines
        })
        .collect();
    lines
}

pub fn get_network_receive_data<'a>(app: &mut App, stats: &Vec<NetworkStats>) -> Vec<Line<'a>> {
    let lines: Vec<Line> = stats
        .iter()
        .map(|interface| {
            let speed = app
                .rx_data
                .get(&interface.name)
                .and_then(|data| speed_kachow(data.clone()))
                .unwrap_or("0 B/s".to_string());

            let sum = interface.receive.bytes + interface.transmit.bytes;
            Line::from(format!(
                " bytes: {}, packets: {}, total: {}, speed: {}",
                interface.receive.display(app, None),
                interface.receive.packets,
                interface.receive.display(app, Some(sum)),
                speed
            ))
        })
        .collect();
    lines
}

pub fn human_speed(bytes_per_sec: f64) -> (f64, &'static str) {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    if bytes_per_sec >= TB {
        (bytes_per_sec / TB, "TB/s")
    } else if bytes_per_sec >= GB {
        (bytes_per_sec / GB, "GB/s")
    } else if bytes_per_sec >= MB {
        (bytes_per_sec / MB, "MB/s")
    } else if bytes_per_sec >= KB {
        (bytes_per_sec / KB, "KB/s")
    } else {
        (bytes_per_sec, "B/s")
    }
}
pub fn speed_kachow(stats: Vec<(f64, f64)>) -> Option<String> {
    if stats.len() < 2 {
        return Some("0 B/s".to_string());
    }

    let (t1, _) = stats[stats.len() - 2];
    let (t2, d2) = stats[stats.len() - 1];
    let dt = t2 - t1;
    let speed = d2 as f64 / dt;
    let (s, string) = human_speed(speed);
    Some(format!("{:.2} {}", s, string))
}

pub fn get_network_transmit_data<'a>(app: &mut App, stats: &Vec<NetworkStats>) -> Vec<Line<'a>> {
    let lines: Vec<Line> = stats
        .iter()
        .map(|interface| {
            let speed = app
                .tx_data
                .get(&interface.name)
                .and_then(|data| speed_kachow(data.clone()))
                .unwrap_or("0 B/s".to_string());

            let sum = interface.receive.bytes + interface.transmit.bytes;
            Line::from(format!(
                " bytes: {}, packets: {}, total: {}, speed: {}",
                interface.transmit.display(app, None),
                interface.transmit.packets,
                interface.transmit.display(app, Some(sum)),
                speed
            ))
        })
        .collect();
    lines
}
pub fn parse_ip_address(s: &str) -> Result<([u8; 4], u16)> {
    let mut s = s.split(":");
    let ip_hex_value = s.next().ok_or(anyhow!("Failed to parse IP"))?;
    let ip_bytes = (0..4)
        .map(|x| u8::from_str_radix(&ip_hex_value[2 * x..2 * x + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    let ip = [ip_bytes[3], ip_bytes[2], ip_bytes[1], ip_bytes[0]];
    let port_hex_value = s.next().ok_or(anyhow!("Failed to parse PORT"))?;
    let port = u16::from_str_radix(port_hex_value, 16)?;
    Ok((ip, port))
}

pub fn parse_hex_values(s: &str) -> Result<u64> {
    let hex_value = u64::from_str_radix(&s, 16)?;
    Ok(hex_value)
}

pub fn parse_hex_value_pairs(s: &str) -> Result<(u64, u64)> {
    let mut s = s.split(":");
    let hex_value_one = u64::from_str_radix(
        s.next().ok_or(anyhow!("Failed to parse left hex value"))?,
        16,
    )?;
    let hex_value_two = u64::from_str_radix(
        s.next().ok_or(anyhow!("Failed to parse right hex value"))?,
        16,
    )?;
    Ok((hex_value_one, hex_value_two))
}

pub fn parse_proc_net_tcp() -> Result<Vec<TcpStats>> {
    let net_tcp_file = PathBuf::from("/proc/net/tcp");
    let mut output = Vec::new();
    if let Err(e) = fs::metadata(&net_tcp_file) {
        eprintln!("{e}");
    } else {
        let file = File::open(&net_tcp_file)?;
        let reader = BufReader::new(file);
        for line in reader.lines().skip(1) {
            let line = line?;
            let first_split: Vec<&str> = line.trim().split_whitespace().collect();

            if first_split.len() < 12 {
                continue;
            }

            let sl = first_split[0].trim_end_matches(":").parse::<u16>()?;

            let (local_ip, local_port) = parse_ip_address(first_split[1])?;
            let (remote_ip, remote_port) = parse_ip_address(first_split[2])?;

            let state = parse_hex_values(first_split[3])?;
            let (tx_queue, rx_queue) = parse_hex_value_pairs(first_split[4])?;
            let (timer_active, timer_when) = parse_hex_value_pairs(first_split[5])?;
            let retrnsmt_timeout = parse_hex_values(first_split[6])?;

            let uid = first_split[7].parse::<u32>()?;
            let timeout = first_split[8].parse::<u32>()?;
            let inode = first_split[9].parse::<u64>()?;

            let values = TcpStats {
                sl: sl,
                local_ip: local_ip,
                local_port: local_port,
                remote_ip: remote_ip,
                remote_port: remote_port,
                state: state,
                tx_queue: tx_queue,
                rx_queue: rx_queue,
                timer_active: timer_active,
                timer_when: timer_when,
                retransmit_timeout: retrnsmt_timeout,
                uid: uid,
                timeout: timeout,
                inode: inode,
            };
            output.push(values);
        }
    }
    Ok(output)
}

pub fn extract_speed(data_str: &str) -> String {
    data_str
        .split("speed: ")
        .nth(1)
        .unwrap_or("0 B/s")
        .trim()
        .to_string()
}

pub fn extract_speed_from_line(line: &Line) -> String {
    let text: String = line
        .spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect();
    extract_speed(&text)
}

pub fn parse_speed(speed_str: &str, link_speed: Option<f64>) -> f64 {
    let val = speed_str
        .split_whitespace()
        .next()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let speed_mbps = if speed_str.contains("GB/s") {
        val * 1000.0
    } else if speed_str.contains("MB/s") {
        val
    } else if speed_str.contains("KB/s") {
        val / 1000.0
    } else {
        0.0
    };

    let Some(link) = link_speed else {
        return speed_mbps;
    };

    (speed_mbps / link).min(1.0)
}

pub fn format_ip(ip: &[u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}

pub fn tcp_state_name(state: u64) -> &'static str {
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

pub fn format_timer(timer_active: u64) -> &'static str {
    match timer_active {
        0 => "off",
        1 => "on",
        2 => "keepalive",
        3 => "timewait",
        4 => "probe",
        _ => "unknown",
    }
}

pub fn format_speed_mbps(mbps: f64) -> String {
    if mbps >= 1000.0 {
        format!("{:.2} GB/s", mbps / 1000.0)
    } else if mbps >= 1.0 {
        format!("{:.2} MB/s", mbps)
    } else {
        format!("{:.2} KB/s", mbps * 1000.0)
    }
}

pub fn parse_uptime() -> Result<String> {
    let mut file = File::open("/proc/uptime")?;
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let uptime_secs = buf
        .split('.')
        .next()
        .expect("Failed to parse uptime")
        .parse::<u64>()
        .expect("Failed to parse uptime");

    let hours = uptime_secs / 3600;
    let minutes = uptime_secs % 3600 / 60;
    let secs = uptime_secs % 60;
    let out_string = format!("{hours}:{minutes}:{secs}");
    Ok(out_string)
}

const CONF_FILE: &'static str = "rxtx.conf";

pub fn initialize_conf() -> Result<(), Error> {
    if !Path::new(CONF_FILE).exists() {
        let theme = format!("Theme: Default\nInterface: default, 0, 0");

        fs::write(CONF_FILE, theme)?;
    }

    Ok(())
}
// Fetches the theme from the conf, if found nothing then defaults to Default theme
pub fn get_theme() -> Theme {
    let default_theme = Theme::default();
    let mut file = match OpenOptions::new().read(true).open(CONF_FILE) {
        std::result::Result::Ok(f) => f,
        Err(_) => return default_theme,
    };
    let mut buf = String::new();
    if file.read_to_string(&mut buf).is_err() {
        return default_theme;
    }

    let mut theme_string = String::new();
    for line in buf.lines() {
        let line: Vec<&str> = line.split(":").collect();
        if line.len() == 2 && line[0] == "Theme" {
            theme_string = line[1]
                .trim()
                .parse::<String>()
                .unwrap_or(String::from("Default"));
            break;
        }
    }

    let theme = THEMES
        .iter()
        .find(|x| x.0.to_lowercase().contains(&theme_string.to_lowercase()));

    match theme {
        Some(t) => t.1(),
        None => default_theme,
    }
}

// Saves the theme
pub fn save_theme(theme: &'static str) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(CONF_FILE)?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)?;

    for line in buf.lines() {
        let line_vec: Vec<&str> = line.split(":").collect();
        if line_vec.contains(&"Theme") && line_vec.len() == 2 {
            let old_theme = line_vec[1];
            let new_theme = buf.replace(old_theme, &format!(" {}", theme));

            if old_theme != new_theme {
                fs::write(CONF_FILE, new_theme)?;
                break;
            }
        }
    }
    Ok(())
}

// Option for user to write to the rxtx.conf file in `Interface: interface, rx_top_speed, tx_top_speed` format and this function will just get the information.
pub fn get_interface_speed() -> HashMap<String, (f64, f64)> {
    let mut map: HashMap<String, (f64, f64)> = HashMap::new();

    let mut file = match OpenOptions::new().read(true).open(CONF_FILE) {
        std::result::Result::Ok(f) => f,
        Err(_) => return map,
    };

    let mut buf = String::new();
    if file.read_to_string(&mut buf).is_err() {
        return map;
    }

    for line in buf.lines() {
        if let Some(s) = line.strip_prefix("Interface: ") {
            let parts: Vec<_> = s.split(',').map(|x| x.trim()).collect();

            if parts.len() == 3 {
                let interface_name = parts[0].trim();
                let rx_speed = parts[1].trim().parse::<f64>().unwrap_or(0.0);
                let tx_speed = parts[2].trim().parse::<f64>().unwrap_or(0.0);

                if !interface_name.is_empty() && rx_speed > 0.0 && tx_speed > 0.0 {
                    map.insert(interface_name.to_string(), (rx_speed, tx_speed));
                }
            } else {
                return map;
            }
        }
    }
    map
}

// Makes Changes to `rxtx.conf` file - those information will be taken from the TUI.
pub fn save_interface_speeds(map: &HashMap<String, (f64, f64)>) -> Result<(), Error> {
    let data = fs::read_to_string(CONF_FILE)?;
    let mut changed = false;

    let mut new_lines = Vec::new();
    let mut seen = HashSet::new();

    for line in data.lines() {
        if let Some(s) = line.strip_prefix("Interface: ") {
            let parts: Vec<_> = s.split(',').map(|x| x.trim()).collect();
            if parts.len() == 3 {
                let name = parts[0];
                let name_lc = name.to_lowercase();

                if let Some((new_name, (rx, tx))) = map
                    .iter()
                    .find(|(n, _)| n.to_lowercase() == name_lc || name_lc == "default")
                {
                    seen.insert(new_name.to_lowercase());
                    changed = true;
                    new_lines.push(format!("Interface: {}, {}, {}", new_name, rx, tx));
                    continue;
                }
            }
        }

        new_lines.push(line.to_string());
    }

    for (name, (rx, tx)) in map {
        let name_lc = name.to_lowercase();
        if !seen.contains(&name_lc) {
            changed = true;
            new_lines.push(format!("Interface: {}, {}, {}", name, rx, tx));
        }
    }

    if changed {
        fs::write(CONF_FILE, new_lines.join("\n"))?;
    }

    Ok(())
}
