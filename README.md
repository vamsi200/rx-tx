# RX-TX

A **Terminal User Interface (TUI)** based bandwidth (`/proc/net/dev`) and network (`/proc/net/tcp`) monitoring tool built for Linux.

# Description
This project is a terminal-based bandwidth and network monitoring tool designed to provide real-time visibility into system and interface-level network activity. 
It presents a unified view of network interfaces, live bandwidth usage, detailed kernel statistics, and active TCP connections, all in a single interactive TUI.

>Note: Don't forget to change the LinkSpeed using `R` for RX Link Speed(Download) and `T` for TX Link Speed(Upload).

# Demo GIF
![TUI demo](./demo/demo.gif)

# TUI Interface Overview

### Global Overview Mode (`all` selected)

When **`all`** is selected in the Interfaces panel, the TUI switches to a system-wide overview:

### Aggregated Metrics
- System uptime
- Total RX / TX across all interfaces
- Total packets
- Total errors
- Total drops
- RX/TX byte ratio
- RX/TX packet ratio
- Drop rate ratio

### Historical Graphs
- RX throughput history (MB/s)
- TX throughput history (MB/s)

These graphs show **aggregated bandwidth trends over time**, giving a quick visual indication of network load and spikes across the entire system.

## Interfaces Panel (Left)

Displays all detected network interfaces, including:

- Physical interfaces (e.g. `enp4s0`, `wlan0`)
- Loopback (`lo`)
- Virtual and bridge interfaces (`docker0`, `br-*`, `veth*`, `lxdbr0`)
- A **`all`** option for system-wide aggregation

Selecting **`all`** switches the UI into a global system overview mode.


## Bandwidth Visualization (Top)

### RX (Receive) and TX (Transmit)

For the selected interface, the TUI shows:

- **Current rate** (bytes/sec)
- **Peak rate**
- **Average rate**
- **Configurable link speed** (used for scaling graphs)
- **Tick interval** (sampling rate)

Bandwidth is visualized using real-time horizontal bar graphs:

- RX and TX are shown independently
- Graphs scale dynamically based on configured link speed
- Direction indicators show trend movement (▲ / ▼)

These graphs are based on **delta sampling** between ticks, not cumulative counters.


## Interface Statistics (Middle)

Detailed kernel-level counters for the selected interface:

### General
- Interface name
- Total traffic (RX + TX)
- RX bytes / packets
- TX bytes / packets

### RX-specific
- Errors
- Drops
- FIFO overruns
- Frame errors
- Multicast packets

### TX-specific
- Errors
- Drops
- Carrier issues
- Collisions
- Compressed packets

This data is read directly from `/sys/class/net/<iface>/statistics`.


## TCP Connections Panel (Bottom)

Displays all active TCP sockets on the system:

- Local address and port
- Remote address and port
- Reverse-resolved hostname (when available)
- Connection state (`LISTEN`, `ESTABLISHED`, etc.)
- TX:RX byte ratio
- UID owning the socket
- Kernel inode number

A summary sidebar shows:

- Total connections
- Active connections
- Unique remote IPs
- Local vs external connections
- State breakdown (ESTABLISHED / LISTEN)

Filtering allows searching across **all fields**, including IPs, hostnames, states, UID, and inode.

  
## Prerequisites

### System Requirements

- **Linux** (kernel with `/proc` and `/sys` support)
- A modern terminal with UTF-8 support

### Development Tools

You will need **Rust** and basic **build tooling** installed.

#### 1. Install Rust

Recommended method (official):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### 2. Install Build Essentials / Development Tools

These are required for compiling native dependencies.

Arch Linux

```bash
sudo pacman -S base-devel
```

Debian / Ubuntu

```bash
sudo apt update
sudo apt install build-essential
```

Fedora

```bash
sudo dnf install @development-tools
```

## Installation

### Build from source

```bash
git clone https://github.com/vamsi200/rx-tx.git
cd rx-tx
cargo build --release
```

## Usage 

Binary will be available at:
```bash
./target/release/rx-tx
```

### Keyboard Shortcuts

### Global

| Key | Action |
|---|---|
| `q` | Quit application |
| `?` | Toggle help menu |
| `Tab` | Switch focus between **Interfaces** and **TCP Connections** |
| `K` | Change tick rate (refresh interval) |


### Interfaces View

| Key | Action |
|---|---|
| `↑ / ↓` | Navigate interface list |
| `Enter` | Select interface / Select **All** |
| `f` | Filter interfaces by name |
| `R` | Edit **RX** speed limit |
| `T` | Edit **TX** speed limit |
| `b` | Toggle byte units (KiB / MiB / GiB ↔ KB / MB / GB) |
| `d` | Toggle decimal / binary units |
| `r` | Toggle raw bytes display |


### TCP Connections View

| Key | Action |
|---|---|
| `↑ / ↓` | Scroll through connections |
| `f` | Filter connections (all fields) |


### Filter Mode

| Key | Action |
|---|---|
| `Type` | Search across all fields (IP, hostname, state, UID, inode) |
| `↑ / ↓` | Navigate filtered results |
| `Enter` | Lock onto selected connection |
| `Enter` (again) | Unlock connection |
| `Esc` | Exit filter mode |

### Help

| Key | Action |
|---|---|
| `?` | Open help |
| `Esc` | Close help |

