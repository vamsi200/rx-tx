#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]
use anyhow::Result;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::ptr::copy_nonoverlapping;
use std::thread;

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

fn parse_data() -> Result<Vec<NetworkStats>> {
    let data_path = PathBuf::from("/proc/net/dev");
    let mut output = Vec::new();

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

fn main() -> Result<()> {
    let output = parse_data()?;

    for data in output {
        if data.name == "wlan0" {
            println!("{:?}", data.receive);
        }
    }

    Ok(())
}
