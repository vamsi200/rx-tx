use ratatui::style::Color;

#[derive(Clone, Debug)]
pub struct CommonColor {
    pub heading: Color,
    pub data: Color,
    pub muted: Color,
    pub tick: Color,
}

#[derive(Clone, Debug)]
pub struct OverviewAreaColor {
    pub border: Color,
    pub tick_heading: Color,
    pub tick_highlight: Color,
    pub tick_value: Color,
    pub key: Color,
    pub val: Color,
}

#[derive(Clone, Debug)]
pub struct RxGraphAreaColor {
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct TxGraphAreaColor {
    pub color: Color,
}

#[derive(Clone, Debug)]
pub struct InterfaceAreaColor {
    pub border: Color,
    pub filter_highlight_symbol: Color,
    pub name: Color,
    pub filter_heading: Color,
    pub filter_background: Color,
}

#[derive(Clone, Debug)]
pub struct RxBarAreaColor {
    pub border: Color,
    pub name: Color,
    pub common_heading: Color,
    pub current_val: Color,
    pub peak_val: Color,
    pub average_val: Color,
    pub link_speed_highlight: Color,
    pub link_speed_heading: Color,
    pub link_speed_val: Color,
    pub tick_highlight: Color,
    pub tick_heading: Color,
    pub tick_val: Color,
}

#[derive(Clone, Debug)]
pub struct TxBarAreaColor {
    pub border: Color,
    pub name: Color,
    pub common_heading: Color,
    pub current_val: Color,
    pub peak_val: Color,
    pub average_val: Color,
    pub link_speed_highlight: Color,
    pub link_speed_heading: Color,
    pub link_speed_val: Color,
}

#[derive(Clone, Debug)]
pub struct InfoAreaColor {
    pub heading: Color,
    pub key: Color,
    pub val: Color,
}

#[derive(Clone, Debug)]
pub struct RxAreaColor {
    pub heading: Color,
    pub key: Color,
    pub val: Color,
}

#[derive(Clone, Debug)]
pub struct SparkLineColor {
    pub rx_border_color: Color,
    pub tx_border_color: Color,
    pub rx_sparkline: Color,
    pub tx_sparkline: Color,
}

#[derive(Clone, Debug)]
pub struct TxAreaColor {
    pub heading: Color,
    pub key: Color,
    pub val: Color,
}

#[derive(Clone, Debug)]
pub struct TcpInfoAreaColor {
    pub heading: Color,
    pub key: Color,
    pub val: Color,
}

#[derive(Clone, Debug)]
pub struct TcpConnAreaColor {
    pub border: Color,
    pub filter_highlight_symbol: Color,
    pub filter: Color,
    pub heading: Color,
    pub local_addr_val: Color,
    pub remote_addr_val: Color,
    pub hostname_val: Color,
    pub txrx_val: Color,
    pub uid_val: Color,
    pub inode_val: Color,
    pub state_established: Color,
    pub state_listen: Color,
    pub state_time_wait: Color,
    pub state_close_wait: Color,
    pub state_syn_sent: Color,
    pub state_fin_wait: Color,
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub interface_area_color: InterfaceAreaColor,
    pub overview_area_color: OverviewAreaColor,
    pub rxgraph_area_color: RxGraphAreaColor,
    pub txgraph_area_color: TxGraphAreaColor,
    pub rxbar_area_color: RxBarAreaColor,
    pub txbar_area_color: TxBarAreaColor,
    pub info_area_color: InfoAreaColor,
    pub rx_area_color: RxAreaColor,
    pub tx_area_color: TxAreaColor,
    pub tcpinfo_area_color: TcpInfoAreaColor,
    pub tcpconn_area_color: TcpConnAreaColor,
    pub sparkline_area_color: SparkLineColor,
}

pub static THEMES: [(&str, fn() -> Theme); 6] = [
    ("Default", Theme::default),
    ("Gruvbox", gruvbox),
    ("Ayu", ayu),
    ("Solarized Dark", solarized_dark),
    ("Catppuccin Mocha", catppuccin_mocha),
    ("Nord", nord),
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
                filter_heading: Color::Red,
                filter_background: Color::Red,
            },

            overview_area_color: OverviewAreaColor {
                tick_highlight: Color::Red,
                tick_heading: common.heading,
                tick_value: Color::Yellow,
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
                filter_highlight_symbol: Color::Red,
                filter: Color::Yellow,
                heading: common.heading,
                local_addr_val: Color::Green,
                remote_addr_val: Color::Blue,
                hostname_val: common.muted,
                txrx_val: common.muted,
                uid_val: common.muted,
                inode_val: common.muted,
                state_established: Color::Green,
                state_listen: Color::Yellow,
                state_time_wait: Color::Rgb(190, 190, 190),
                state_close_wait: Color::Rgb(200, 140, 60),
                state_syn_sent: Color::Rgb(220, 180, 60),
                state_fin_wait: Color::Rgb(180, 100, 140),
            },

            sparkline_area_color: SparkLineColor {
                rx_border_color: Color::Green,
                tx_border_color: Color::Blue,
                rx_sparkline: Color::Green,
                tx_sparkline: Color::Blue,
            },
        }
    }
}

fn nord() -> Theme {
    let common = CommonColor {
        heading: Color::Rgb(76, 86, 106),
        data: Color::Rgb(216, 222, 233),
        muted: Color::Rgb(129, 161, 193),
        tick: Color::Rgb(59, 66, 82),
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Rgb(129, 161, 193),
            filter_highlight_symbol: Color::Rgb(150, 210, 230),

            name: Color::Rgb(129, 161, 193),
            filter_heading: Color::Rgb(94, 129, 172),
            filter_background: Color::Rgb(129, 161, 193),
        },

        overview_area_color: OverviewAreaColor {
            tick_highlight: Color::Rgb(136, 192, 208),
            tick_heading: common.heading,
            tick_value: Color::Rgb(129, 161, 193),
            border: Color::Rgb(129, 161, 193),
            key: common.heading,
            val: common.data,
        },

        rxgraph_area_color: RxGraphAreaColor {
            color: Color::Rgb(143, 188, 187),
        },

        txgraph_area_color: TxGraphAreaColor {
            color: Color::Rgb(94, 129, 172),
        },

        rxbar_area_color: RxBarAreaColor {
            border: Color::Rgb(143, 188, 187),
            name: Color::Rgb(143, 188, 187),
            common_heading: common.heading,
            current_val: Color::Rgb(94, 129, 172),
            peak_val: Color::Rgb(143, 188, 187),
            average_val: Color::Rgb(94, 129, 172),
            link_speed_highlight: Color::Rgb(129, 161, 193),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(143, 188, 187),
            tick_highlight: Color::Rgb(136, 192, 208),
            tick_heading: common.heading,
            tick_val: Color::Rgb(94, 129, 172),
        },

        txbar_area_color: TxBarAreaColor {
            border: Color::Rgb(94, 129, 172),
            name: Color::Rgb(94, 129, 172),
            common_heading: common.heading,
            current_val: Color::Rgb(143, 188, 187),
            peak_val: Color::Rgb(94, 129, 172),
            average_val: Color::Rgb(143, 188, 187),
            link_speed_highlight: Color::Rgb(129, 161, 193),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(94, 129, 172),
        },

        info_area_color: InfoAreaColor {
            heading: Color::Rgb(136, 192, 208),
            key: common.heading,
            val: common.data,
        },

        rx_area_color: RxAreaColor {
            heading: Color::Rgb(143, 188, 187),
            key: common.heading,
            val: common.data,
        },

        tx_area_color: TxAreaColor {
            heading: Color::Rgb(94, 129, 172),
            key: common.heading,
            val: common.data,
        },

        tcpinfo_area_color: TcpInfoAreaColor {
            heading: Color::Rgb(143, 188, 187),
            key: common.heading,
            val: common.data,
        },

        tcpconn_area_color: TcpConnAreaColor {
            border: Color::Rgb(129, 161, 193),
            filter_highlight_symbol: Color::Rgb(129, 161, 193),
            filter: Color::Rgb(136, 192, 208),
            heading: common.heading,
            local_addr_val: Color::Rgb(143, 188, 187),
            remote_addr_val: Color::Rgb(94, 129, 172),
            hostname_val: common.muted,
            txrx_val: common.muted,
            uid_val: common.muted,
            inode_val: common.muted,
            state_established: Color::Rgb(143, 188, 187),
            state_listen: Color::Rgb(136, 192, 208),
            state_time_wait: Color::Rgb(76, 86, 106),
            state_close_wait: Color::Rgb(129, 161, 193),
            state_syn_sent: Color::Rgb(136, 192, 208),
            state_fin_wait: Color::Rgb(94, 129, 172),
        },

        sparkline_area_color: SparkLineColor {
            rx_border_color: Color::Rgb(143, 188, 187),
            tx_border_color: Color::Rgb(94, 129, 172),
            rx_sparkline: Color::Rgb(143, 188, 187),
            tx_sparkline: Color::Rgb(94, 129, 172),
        },
    }
}

fn gruvbox() -> Theme {
    let common = CommonColor {
        heading: Color::Rgb(124, 111, 100),
        data: Color::Rgb(235, 219, 178),
        muted: Color::Rgb(168, 153, 132),
        tick: Color::Rgb(102, 92, 84),
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Rgb(204, 36, 29),
            filter_highlight_symbol: Color::Rgb(250, 189, 47),
            name: Color::Rgb(251, 73, 52),
            filter_heading: Color::Rgb(251, 73, 52),
            filter_background: Color::Rgb(204, 36, 29),
        },

        overview_area_color: OverviewAreaColor {
            tick_highlight: Color::Rgb(251, 73, 52),
            tick_heading: common.heading,
            tick_value: Color::Rgb(215, 153, 33),
            border: Color::Rgb(215, 153, 33),
            key: common.heading,
            val: common.data,
        },

        rxgraph_area_color: RxGraphAreaColor {
            color: Color::Rgb(142, 192, 124),
        },

        txgraph_area_color: TxGraphAreaColor {
            color: Color::Rgb(131, 165, 152),
        },

        rxbar_area_color: RxBarAreaColor {
            border: Color::Rgb(184, 187, 38),
            name: Color::Rgb(184, 187, 38),
            common_heading: common.heading,
            current_val: Color::Rgb(131, 165, 152),
            peak_val: Color::Rgb(184, 187, 38),
            average_val: Color::Rgb(131, 165, 152),
            link_speed_highlight: Color::Rgb(152, 151, 26),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(184, 187, 38),
            tick_highlight: Color::Rgb(251, 73, 52),
            tick_heading: common.heading,
            tick_val: Color::Rgb(131, 165, 152),
        },

        txbar_area_color: TxBarAreaColor {
            border: Color::Rgb(131, 165, 152),
            name: Color::Rgb(131, 165, 152),
            common_heading: common.heading,
            current_val: Color::Rgb(184, 187, 38),
            peak_val: Color::Rgb(131, 165, 152),
            average_val: Color::Rgb(184, 187, 38),
            link_speed_highlight: Color::Rgb(152, 151, 26),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(131, 165, 152),
        },

        info_area_color: InfoAreaColor {
            heading: Color::Rgb(250, 189, 47),
            key: common.heading,
            val: common.data,
        },

        rx_area_color: RxAreaColor {
            heading: Color::Rgb(142, 192, 124),
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
            border: Color::Rgb(131, 165, 152),
            filter_highlight_symbol: Color::Rgb(204, 36, 29),
            filter: Color::Rgb(250, 189, 47),
            heading: common.heading,
            local_addr_val: Color::Rgb(184, 187, 38),
            remote_addr_val: Color::Rgb(131, 165, 152),
            hostname_val: common.muted,
            txrx_val: common.muted,
            uid_val: common.muted,
            inode_val: common.muted,
            state_established: Color::Rgb(184, 187, 38),
            state_listen: Color::Rgb(250, 189, 47),
            state_time_wait: Color::Rgb(146, 131, 116),
            state_close_wait: Color::Rgb(254, 128, 25),
            state_syn_sent: Color::Rgb(215, 153, 33),
            state_fin_wait: Color::Rgb(211, 134, 155),
        },

        sparkline_area_color: SparkLineColor {
            rx_border_color: Color::Rgb(142, 192, 124),
            tx_border_color: Color::Rgb(131, 165, 152),
            rx_sparkline: Color::Rgb(104, 157, 106),
            tx_sparkline: Color::Rgb(69, 133, 136),
        },
    }
}

fn solarized_dark() -> Theme {
    let common = CommonColor {
        heading: Color::Rgb(88, 110, 117),
        data: Color::Rgb(131, 148, 150),
        muted: Color::Rgb(101, 123, 131),
        tick: Color::Rgb(7, 54, 66),
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Rgb(108, 113, 196),
            filter_highlight_symbol: Color::Rgb(220, 50, 47),
            name: Color::Rgb(108, 113, 196),
            filter_heading: Color::Rgb(181, 137, 0),
            filter_background: Color::Rgb(108, 113, 196),
        },

        overview_area_color: OverviewAreaColor {
            tick_highlight: Color::Rgb(220, 50, 47),
            tick_heading: common.heading,
            tick_value: Color::Rgb(181, 137, 0),
            border: Color::Rgb(181, 137, 0),
            key: common.heading,
            val: common.data,
        },

        rxgraph_area_color: RxGraphAreaColor {
            color: Color::Rgb(42, 161, 152),
        },

        txgraph_area_color: TxGraphAreaColor {
            color: Color::Rgb(133, 153, 0),
        },

        rxbar_area_color: RxBarAreaColor {
            border: Color::Rgb(42, 161, 152),
            name: Color::Rgb(42, 161, 152),
            common_heading: common.heading,
            current_val: Color::Rgb(133, 153, 0),
            peak_val: Color::Rgb(42, 161, 152),
            average_val: Color::Rgb(133, 153, 0),
            link_speed_highlight: Color::Rgb(42, 161, 152),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(133, 153, 0),
            tick_highlight: Color::Rgb(220, 50, 47),
            tick_heading: common.heading,
            tick_val: Color::Rgb(42, 161, 152),
        },
        txbar_area_color: TxBarAreaColor {
            border: Color::Rgb(133, 153, 0),
            name: Color::Rgb(133, 153, 0),
            common_heading: common.heading,
            current_val: Color::Rgb(42, 161, 152),
            peak_val: Color::Rgb(133, 153, 0),
            average_val: Color::Rgb(42, 161, 152),
            link_speed_highlight: Color::Rgb(133, 153, 0),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(42, 161, 152),
        },

        info_area_color: InfoAreaColor {
            heading: Color::Rgb(181, 137, 0),
            key: common.heading,
            val: common.data,
        },

        rx_area_color: RxAreaColor {
            heading: Color::Rgb(42, 161, 152),
            key: common.heading,
            val: common.data,
        },

        tx_area_color: TxAreaColor {
            heading: Color::Rgb(133, 153, 0),
            key: common.heading,
            val: common.data,
        },

        tcpinfo_area_color: TcpInfoAreaColor {
            heading: Color::Rgb(133, 153, 0),
            key: common.heading,
            val: common.data,
        },

        tcpconn_area_color: TcpConnAreaColor {
            border: Color::Rgb(38, 139, 210),
            filter_highlight_symbol: Color::Rgb(108, 113, 196),
            filter: Color::Rgb(220, 50, 47),
            heading: common.heading,
            local_addr_val: Color::Rgb(133, 153, 0),
            remote_addr_val: Color::Rgb(38, 139, 210),
            hostname_val: common.muted,
            txrx_val: common.muted,
            uid_val: common.muted,
            inode_val: common.muted,
            state_established: Color::Rgb(133, 153, 0),
            state_listen: Color::Rgb(181, 137, 0),
            state_time_wait: Color::Rgb(88, 110, 117),
            state_close_wait: Color::Rgb(203, 75, 22),
            state_syn_sent: Color::Rgb(181, 137, 0),
            state_fin_wait: Color::Rgb(108, 113, 196),
        },

        sparkline_area_color: SparkLineColor {
            rx_border_color: Color::Rgb(42, 161, 152),
            tx_border_color: Color::Rgb(133, 153, 0),
            rx_sparkline: Color::Rgb(42, 161, 152),
            tx_sparkline: Color::Rgb(133, 153, 0),
        },
    }
}

fn catppuccin_mocha() -> Theme {
    let common = CommonColor {
        heading: Color::Rgb(108, 112, 134),
        data: Color::Rgb(205, 214, 244),
        muted: Color::Rgb(147, 153, 178),
        tick: Color::Rgb(88, 91, 112),
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Rgb(203, 166, 247),
            filter_highlight_symbol: Color::Rgb(250, 179, 135),
            name: Color::Rgb(203, 166, 247),
            filter_heading: Color::Rgb(243, 139, 168),
            filter_background: Color::Rgb(203, 166, 247),
        },

        overview_area_color: OverviewAreaColor {
            tick_highlight: Color::Rgb(243, 139, 168),
            tick_heading: common.heading,
            tick_value: Color::Rgb(249, 226, 175),
            border: Color::Rgb(249, 226, 175),
            key: common.heading,
            val: common.data,
        },

        rxgraph_area_color: RxGraphAreaColor {
            color: Color::Rgb(166, 227, 161),
        },

        txgraph_area_color: TxGraphAreaColor {
            color: Color::Rgb(203, 166, 247),
        },

        rxbar_area_color: RxBarAreaColor {
            border: Color::Rgb(166, 227, 161),
            name: Color::Rgb(166, 227, 161),
            common_heading: common.heading,
            current_val: Color::Rgb(203, 166, 247),
            peak_val: Color::Rgb(166, 227, 161),
            average_val: Color::Rgb(203, 166, 247),
            link_speed_highlight: Color::Rgb(148, 226, 213),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(166, 227, 161),
            tick_highlight: Color::Rgb(250, 179, 135),
            tick_heading: common.heading,
            tick_val: Color::Rgb(203, 166, 247),
        },

        txbar_area_color: TxBarAreaColor {
            border: Color::Rgb(203, 166, 247),
            name: Color::Rgb(203, 166, 247),
            common_heading: common.heading,
            current_val: Color::Rgb(166, 227, 161),
            peak_val: Color::Rgb(203, 166, 247),
            average_val: Color::Rgb(166, 227, 161),
            link_speed_highlight: Color::Rgb(148, 226, 213),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(203, 166, 247),
        },

        info_area_color: InfoAreaColor {
            heading: Color::Rgb(249, 226, 175),
            key: common.heading,
            val: common.data,
        },

        rx_area_color: RxAreaColor {
            heading: Color::Rgb(166, 227, 161),
            key: common.heading,
            val: common.data,
        },

        tx_area_color: TxAreaColor {
            heading: Color::Rgb(203, 166, 247),
            key: common.heading,
            val: common.data,
        },

        tcpinfo_area_color: TcpInfoAreaColor {
            heading: Color::Rgb(166, 227, 161),
            key: common.heading,
            val: common.data,
        },

        tcpconn_area_color: TcpConnAreaColor {
            border: Color::Rgb(137, 180, 250),
            filter_highlight_symbol: Color::Rgb(203, 166, 247),
            filter: Color::Rgb(250, 179, 135),
            heading: common.heading,
            local_addr_val: Color::Rgb(166, 227, 161),
            remote_addr_val: Color::Rgb(137, 180, 250),
            hostname_val: common.muted,
            txrx_val: common.muted,
            uid_val: common.muted,
            inode_val: common.muted,
            state_established: Color::Rgb(166, 227, 161),
            state_listen: Color::Rgb(249, 226, 175),
            state_time_wait: Color::Rgb(147, 153, 178),
            state_close_wait: Color::Rgb(250, 179, 135),
            state_syn_sent: Color::Rgb(249, 226, 175),
            state_fin_wait: Color::Rgb(245, 194, 231),
        },

        sparkline_area_color: SparkLineColor {
            rx_border_color: Color::Rgb(166, 227, 161),
            tx_border_color: Color::Rgb(203, 166, 247),
            rx_sparkline: Color::Rgb(166, 227, 161),
            tx_sparkline: Color::Rgb(203, 166, 247),
        },
    }
}

fn ayu() -> Theme {
    let common = CommonColor {
        heading: Color::Rgb(92, 99, 112),
        data: Color::Rgb(230, 230, 230),
        muted: Color::Rgb(92, 99, 112),
        tick: Color::Rgb(61, 67, 81),
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Rgb(242, 151, 24),
            filter_highlight_symbol: Color::Rgb(255, 180, 84),
            name: Color::Rgb(242, 151, 24),
            filter_heading: Color::Rgb(220, 50, 47),
            filter_background: Color::Rgb(191, 97, 106),
        },

        overview_area_color: OverviewAreaColor {
            tick_highlight: Color::Red,
            tick_heading: Color::DarkGray,
            tick_value: Color::Yellow,

            border: Color::Rgb(255, 180, 84),
            key: common.heading,
            val: common.data,
        },

        rxgraph_area_color: RxGraphAreaColor {
            color: Color::Rgb(185, 202, 74),
        },

        txgraph_area_color: TxGraphAreaColor {
            color: Color::Rgb(57, 186, 230),
        },

        rxbar_area_color: RxBarAreaColor {
            border: Color::Rgb(185, 202, 74),
            name: Color::Rgb(185, 202, 74),
            common_heading: common.heading,
            current_val: Color::Rgb(57, 186, 230),
            peak_val: Color::Rgb(185, 202, 74),
            average_val: Color::Rgb(95, 187, 187),
            link_speed_highlight: Color::Rgb(57, 186, 230),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(185, 202, 74),
            tick_highlight: Color::Rgb(242, 151, 24),
            tick_heading: common.heading,
            tick_val: Color::Rgb(57, 186, 230),
        },

        txbar_area_color: TxBarAreaColor {
            border: Color::Rgb(57, 186, 230),
            name: Color::Rgb(57, 186, 230),
            common_heading: common.heading,
            current_val: Color::Rgb(185, 202, 74),
            peak_val: Color::Rgb(57, 186, 230),
            average_val: Color::Rgb(95, 187, 187),
            link_speed_highlight: Color::Rgb(185, 202, 74),
            link_speed_heading: common.heading,
            link_speed_val: Color::Rgb(57, 186, 230),
        },

        info_area_color: InfoAreaColor {
            heading: Color::Rgb(255, 180, 84),
            key: common.heading,
            val: common.data,
        },

        rx_area_color: RxAreaColor {
            heading: Color::Rgb(185, 202, 74),
            key: common.heading,
            val: common.data,
        },

        tx_area_color: TxAreaColor {
            heading: Color::Rgb(57, 186, 230),
            key: common.heading,
            val: common.data,
        },

        tcpinfo_area_color: TcpInfoAreaColor {
            heading: Color::Rgb(185, 202, 74),
            key: common.heading,
            val: common.data,
        },

        tcpconn_area_color: TcpConnAreaColor {
            border: Color::Rgb(57, 186, 230),
            filter_highlight_symbol: Color::Red,
            filter: Color::Yellow,

            heading: common.heading,
            local_addr_val: Color::Rgb(185, 202, 74),
            remote_addr_val: Color::Rgb(57, 186, 230),
            hostname_val: common.muted,
            txrx_val: common.muted,
            uid_val: common.muted,
            inode_val: common.muted,
            state_established: Color::Green,
            state_listen: Color::Yellow,
            state_time_wait: Color::Rgb(190, 190, 190),
            state_close_wait: Color::Rgb(200, 140, 60),
            state_syn_sent: Color::Rgb(220, 180, 60),
            state_fin_wait: Color::Rgb(180, 100, 140),
        },
        sparkline_area_color: SparkLineColor {
            rx_border_color: Color::Green,
            tx_border_color: Color::Blue,
            rx_sparkline: Color::Green,
            tx_sparkline: Color::Blue,
        },
    }
}
