#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use anyhow::{anyhow, Error, Ok, Result};
use core::net;
use ratatui::crossterm::event::read;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::ptr::copy_nonoverlapping;
use std::thread::{self, sleep};
use std::time::Duration;

#[derive(Debug)]
struct NetworkStats {
    name: String,
    receive: Receive,
    transmit: Transmit,
}

#[derive(Debug)]
struct Receive {
    bytes: u64,
    packets: u64,
    errs: u64,
    drop: u64,
    fifo: u64,
    frame: u64,
    compressed: u64,
    multicast: u64,
}

#[derive(Debug)]
struct Transmit {
    bytes: u64,
    packets: u64,
    errs: u64,
    drop: u64,
    fifo: u64,
    colls: u64,
    carrier: u64,
    compressed: u64,
}

fn parse_proc_net_dev() -> Result<Vec<NetworkStats>> {
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

fn print_proc_net_dev(info: Vec<NetworkStats>) -> Result<()> {
    todo!();
}

#[derive(Debug)]
struct TcpStats {
    sl: u16,
    local_ip: [u8; 4],
    local_port: u16,
    remote_ip: [u8; 4],
    remote_port: u16,
    state: u64,
    tx_queue: u64,
    rx_queue: u64,
    timer_active: u64,
    timer_when: u64,
    retransmit_timeout: u64,
    uid: u32,
    timeout: u32,
    inode: u64,
}

fn parse_ip_address(s: &str) -> Result<([u8; 4], u16)> {
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

fn parse_hex_values(s: &str) -> Result<u64> {
    let hex_value = u64::from_str_radix(&s, 16)?;
    Ok(hex_value)
}

fn parse_hex_value_pairs(s: &str) -> Result<(u64, u64)> {
    let mut s = s.split(":");
    let hex_value_one =
        u64::from_str_radix(s.next().ok_or(anyhow!("Failed to parse left value"))?, 16)?;
    let hex_value_two =
        u64::from_str_radix(s.next().ok_or(anyhow!("Failed to parse right value"))?, 16)?;
    Ok((hex_value_one, hex_value_two))
}

fn parse_proc_net_tcp() -> Result<Vec<TcpStats>> {
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

fn print_proc_net_tcp() -> Result<()> {
    todo!();
}

fn main() -> Result<()> {
    // let interval = Duration::new(2, 0);
    // loop {
    //     let output = parse_proc_net_dev()?;
    //     for data in output {
    //         if data.name == "wlan0" {
    //             println!("{:?}", data.receive);
    //         }
    //     }
    //     sleep(interval);
    // }
    let s = parse_proc_net_tcp()?;
    println!("{:?}", s);
    Ok(())
}
