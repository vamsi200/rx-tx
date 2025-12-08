use crate::app::App;
use crate::models::{self, *};
use anyhow::{anyhow, Error, Ok, Result};
use core::net;
use crossterm::event::{self, Event};
use ratatui::crossterm::event::read;
use ratatui::style::{Modifier, Style};
use ratatui::symbols::line;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::Terminal;
use ratatui::{text::Text, Frame};
use std::fmt::format;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
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

pub fn get_network_interfaces(stats: &Vec<NetworkStats>) -> Vec<Line> {
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
