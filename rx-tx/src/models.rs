use crate::app::{App, ByteUnit};

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub name: String,
    pub receive: Receive,
    pub transmit: Transmit,
}

#[derive(Debug, Clone)]
pub struct Receive {
    pub bytes: u64,
    pub packets: u64,
    pub errs: u64,
    pub drop: u64,
    pub fifo: u64,
    pub frame: u64,
    pub compressed: u64,
    pub multicast: u64,
}

#[derive(Debug, Clone)]
pub struct Transmit {
    pub bytes: u64,
    pub packets: u64,
    pub errs: u64,
    pub drop: u64,
    pub fifo: u64,
    pub colls: u64,
    pub carrier: u64,
    pub compressed: u64,
}

impl Receive {
    pub fn display(&self, raw_bytes: bool, app: &mut App) -> String {
        if raw_bytes {
            self.bytes.to_string()
        } else {
            format_bytes(self.bytes, &app.byte_unit)
        }
    }
}

impl Transmit {
    pub fn display(&self, raw_bytes: bool, app: &mut App) -> String {
        if raw_bytes {
            self.bytes.to_string()
        } else {
            format_bytes(self.bytes, &app.byte_unit)
        }
    }
}

pub fn format_bytes(data: u64, unit: &ByteUnit) -> String {
    match unit {
        ByteUnit::Binary => {
            const KB: f64 = 1024.0;
            const MB: f64 = KB * 1024.0;
            const GB: f64 = MB * 1024.0;
            const TB: f64 = GB * 1024.0;

            let data = data as f64;
            if data >= TB {
                format!("{:.2} TiB", data / TB)
            } else if data >= GB {
                format!("{:.2} GiB", data / GB)
            } else if data >= MB {
                format!("{:.2} MiB", data / MB)
            } else if data >= KB {
                format!("{:.2} KiB", data / KB)
            } else {
                format!("{} B", data as u64)
            }
        }
        ByteUnit::Decimal => {
            let data = data as f64;
            if data >= 1e12 {
                format!("{:.2} TB", data / 1e12)
            } else if data >= 1e9 {
                format!("{:.2} GB", data / 1e9)
            } else if data >= 1e6 {
                format!("{:.2} MB", data / 1e6)
            } else if data >= 1e3 {
                format!("{:.2} KB", data / 1e3)
            } else {
                format!("{} B", data as u64)
            }
        }
    }
}

#[derive(Debug)]
pub struct TcpStats {
    pub sl: u16,
    pub local_ip: [u8; 4],
    pub local_port: u16,
    pub remote_ip: [u8; 4],
    pub remote_port: u16,
    pub state: u64,
    pub tx_queue: u64,
    pub rx_queue: u64,
    pub timer_active: u64,
    pub timer_when: u64,
    pub retransmit_timeout: u64,
    pub uid: u32,
    pub timeout: u32,
    pub inode: u64,
}
