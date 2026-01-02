use ratatui::style::Color;

pub struct CommonColor {
    pub heading: Color,
    pub data: Color,
    pub muted: Color,
}

pub struct OverviewAreaColor {
    pub border: Color,
    pub tick_heading: Color,
    pub tick_highlight: Color,
    pub key: Color,
    pub val: Color,
}

pub struct InterfaceAreaColor {
    pub border: Color,
    pub filter_highlight_symbol: Color,
    pub filter_heading: Color,
    pub filter_background: Color,
    pub data: Color,
}

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
}

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

pub struct InfoAreaColor {
    pub heading: Color,
    pub key: Color,
    pub val: Color,
}

pub struct RxAreaColor {
    pub heading: Color,
    pub key: Color,
    pub val: Color,
}

pub struct SparkLineColor {
    pub rx_border_color: Color,
    pub tx_border_color: Color,
    pub rx_sparkline: Color,
    pub tx_sparkline: Color,
}

pub struct TxAreaColor {
    pub heading: Color,
    pub key: Color,
    pub val: Color,
}

pub struct TcpInfoAreaColor {
    pub heading: Color,
    pub key: Color,
    pub val: Color,
}

pub struct TcpConnAreaColor {
    pub border: Color,
    pub filter_highlight_symbol: Color,
    pub filter: Color,
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

pub struct HelpPopupColor {
    pub border: Color,
    pub global_heading: Color,
    pub interface_heading: Color,
    pub tcp_conn_heading: Color,
    pub filter_mode: Color,
    pub key: Color,
    pub val: Color,
}

pub struct RxPopupColor {
    pub border: Color,
    pub interface_heading: Color,
    pub download_heading: Color,
}

pub struct TxPopupColor {
    pub border: Color,
    pub interface_heading: Color,
    pub download_heading: Color,
}

pub struct Theme {
    pub interface_area_color: InterfaceAreaColor,
    pub overview_area_color: OverviewAreaColor,
    pub rxbar_area_color: RxBarAreaColor,
    pub txbar_area_color: TxBarAreaColor,
    pub info_area_color: InfoAreaColor,
    pub rx_area_color: RxAreaColor,
    pub tx_area_color: TxAreaColor,
    pub tcpinfo_area_color: TcpInfoAreaColor,
    pub tcpconn_area_color: TcpConnAreaColor,
    pub sparkline_area_color: SparkLineColor,
    pub help_popup_color: HelpPopupColor,
    pub rx_popup_color: RxPopupColor,
    pub tx_popup_color: TxPopupColor,
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
        };

        Self {
            interface_area_color: InterfaceAreaColor {
                border: Color::Red,
                filter_highlight_symbol: Color::Yellow,
                filter_heading: Color::Red,
                filter_background: Color::Red,
                data: common.data,
            },

            overview_area_color: OverviewAreaColor {
                tick_highlight: Color::Red,
                tick_heading: common.heading,
                border: Color::Yellow,
                key: common.heading,
                val: common.data,
            },

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

            help_popup_color: HelpPopupColor {
                border: Color::Yellow,
                global_heading: Color::Yellow,
                interface_heading: Color::Green,
                tcp_conn_heading: Color::Blue,
                filter_mode: Color::Magenta,
                key: common.heading,
                val: common.data,
            },
            rx_popup_color: RxPopupColor {
                border: Color::Green,
                interface_heading: Color::Cyan,
                download_heading: Color::Green,
            },
            tx_popup_color: TxPopupColor {
                border: Color::Blue,
                interface_heading: Color::Cyan,
                download_heading: Color::Blue,
            },
        }
    }
}

fn nord() -> Theme {
    let common = CommonColor {
        heading: Color::Rgb(76, 86, 106),
        data: Color::Rgb(216, 222, 233),
        muted: Color::Rgb(129, 161, 193),
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Rgb(129, 161, 193),
            filter_highlight_symbol: Color::Rgb(150, 210, 230),
            data: common.data,
            filter_heading: Color::Rgb(94, 129, 172),
            filter_background: Color::Rgb(129, 161, 193),
        },

        overview_area_color: OverviewAreaColor {
            tick_highlight: Color::Rgb(136, 192, 208),
            tick_heading: common.heading,
            border: Color::Rgb(129, 161, 193),
            key: common.heading,
            val: common.data,
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
        help_popup_color: HelpPopupColor {
            border: Color::Rgb(136, 192, 208),
            global_heading: Color::Rgb(136, 192, 208),
            interface_heading: Color::Rgb(143, 188, 187),
            tcp_conn_heading: Color::Rgb(94, 129, 172),
            filter_mode: Color::Rgb(180, 142, 173),
            key: common.heading,
            val: common.data,
        },
        rx_popup_color: RxPopupColor {
            border: Color::Rgb(143, 188, 187),
            interface_heading: Color::Rgb(136, 192, 208),
            download_heading: Color::Rgb(143, 188, 187),
        },
        tx_popup_color: TxPopupColor {
            border: Color::Rgb(94, 129, 172),
            interface_heading: Color::Rgb(136, 192, 208),
            download_heading: Color::Rgb(94, 129, 172),
        },
    }
}

fn gruvbox() -> Theme {
    let common = CommonColor {
        heading: Color::Rgb(124, 111, 100),
        data: Color::Rgb(235, 219, 178),
        muted: Color::Rgb(168, 153, 132),
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Rgb(204, 36, 29),
            filter_highlight_symbol: Color::Rgb(250, 189, 47),
            filter_heading: Color::Rgb(251, 73, 52),
            filter_background: Color::Rgb(204, 36, 29),
            data: common.data,
        },

        overview_area_color: OverviewAreaColor {
            tick_highlight: Color::Rgb(251, 73, 52),
            tick_heading: common.heading,
            border: Color::Rgb(215, 153, 33),
            key: common.heading,
            val: common.data,
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
        help_popup_color: HelpPopupColor {
            border: Color::Rgb(250, 189, 47),
            global_heading: Color::Rgb(250, 189, 47),
            interface_heading: Color::Rgb(142, 192, 124),
            tcp_conn_heading: Color::Rgb(131, 165, 152),
            filter_mode: Color::Rgb(211, 134, 155),
            key: common.heading,
            val: common.data,
        },
        rx_popup_color: RxPopupColor {
            border: Color::Rgb(142, 192, 124),
            interface_heading: Color::Rgb(104, 157, 106),
            download_heading: Color::Rgb(142, 192, 124),
        },
        tx_popup_color: TxPopupColor {
            border: Color::Rgb(131, 165, 152),
            interface_heading: Color::Rgb(104, 157, 106),
            download_heading: Color::Rgb(131, 165, 152),
        },
    }
}

fn solarized_dark() -> Theme {
    let common = CommonColor {
        heading: Color::Rgb(88, 110, 117),
        data: Color::Rgb(131, 148, 150),
        muted: Color::Rgb(101, 123, 131),
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Rgb(108, 113, 196),
            filter_highlight_symbol: Color::Rgb(220, 50, 47),
            filter_heading: Color::Rgb(181, 137, 0),
            filter_background: Color::Rgb(108, 113, 196),
            data: common.data,
        },

        overview_area_color: OverviewAreaColor {
            tick_highlight: Color::Rgb(220, 50, 47),
            tick_heading: common.heading,
            border: Color::Rgb(181, 137, 0),
            key: common.heading,
            val: common.data,
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

        help_popup_color: HelpPopupColor {
            border: Color::Rgb(181, 137, 0),
            global_heading: Color::Rgb(181, 137, 0),
            interface_heading: Color::Rgb(42, 161, 152),
            tcp_conn_heading: Color::Rgb(38, 139, 210),
            filter_mode: Color::Rgb(211, 54, 130),
            key: common.heading,
            val: common.data,
        },
        rx_popup_color: RxPopupColor {
            border: Color::Rgb(42, 161, 152),
            interface_heading: Color::Rgb(42, 161, 152),
            download_heading: Color::Rgb(42, 161, 152),
        },
        tx_popup_color: TxPopupColor {
            border: Color::Rgb(133, 153, 0),
            interface_heading: Color::Rgb(42, 161, 152),
            download_heading: Color::Rgb(133, 153, 0),
        },
    }
}

fn catppuccin_mocha() -> Theme {
    let common = CommonColor {
        heading: Color::Rgb(108, 112, 134),
        data: Color::Rgb(205, 214, 244),
        muted: Color::Rgb(147, 153, 178),
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Rgb(203, 166, 247),
            filter_highlight_symbol: Color::Rgb(250, 179, 135),
            filter_heading: Color::Rgb(243, 139, 168),
            filter_background: Color::Rgb(203, 166, 247),
            data: common.data,
        },

        overview_area_color: OverviewAreaColor {
            tick_highlight: Color::Rgb(243, 139, 168),
            tick_heading: common.heading,
            border: Color::Rgb(249, 226, 175),
            key: common.heading,
            val: common.data,
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

        help_popup_color: HelpPopupColor {
            border: Color::Rgb(249, 226, 175),
            global_heading: Color::Rgb(249, 226, 175),
            interface_heading: Color::Rgb(166, 227, 161),
            tcp_conn_heading: Color::Rgb(137, 180, 250),
            filter_mode: Color::Rgb(245, 194, 231),
            key: common.heading,
            val: common.data,
        },

        rx_popup_color: RxPopupColor {
            border: Color::Rgb(166, 227, 161),
            interface_heading: Color::Rgb(148, 226, 213),
            download_heading: Color::Rgb(166, 227, 161),
        },
        tx_popup_color: TxPopupColor {
            border: Color::Rgb(203, 166, 247),
            interface_heading: Color::Rgb(148, 226, 213),
            download_heading: Color::Rgb(203, 166, 247),
        },
    }
}

fn ayu() -> Theme {
    let common = CommonColor {
        heading: Color::Rgb(92, 99, 112),
        data: Color::Rgb(230, 230, 230),
        muted: Color::Rgb(92, 99, 112),
    };

    Theme {
        interface_area_color: InterfaceAreaColor {
            border: Color::Rgb(242, 151, 24),
            filter_highlight_symbol: Color::Rgb(255, 180, 84),
            filter_heading: Color::Rgb(220, 50, 47),
            filter_background: Color::Rgb(191, 97, 106),
            data: common.data,
        },

        overview_area_color: OverviewAreaColor {
            tick_highlight: Color::Red,
            tick_heading: Color::DarkGray,

            border: Color::Rgb(255, 180, 84),
            key: common.heading,
            val: common.data,
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

        help_popup_color: HelpPopupColor {
            border: Color::Rgb(255, 180, 84),
            global_heading: Color::Rgb(255, 180, 84),
            interface_heading: Color::Rgb(185, 202, 74),
            tcp_conn_heading: Color::Rgb(57, 186, 230),
            filter_mode: Color::Rgb(242, 151, 24),
            key: common.heading,
            val: common.data,
        },
        rx_popup_color: RxPopupColor {
            border: Color::Rgb(185, 202, 74),
            interface_heading: Color::Rgb(95, 187, 187),
            download_heading: Color::Rgb(185, 202, 74),
        },
        tx_popup_color: TxPopupColor {
            border: Color::Rgb(57, 186, 230),
            interface_heading: Color::Rgb(95, 187, 187),
            download_heading: Color::Rgb(57, 186, 230),
        },
    }
}
