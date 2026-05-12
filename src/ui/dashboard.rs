use iced::widget::{
    column, container, horizontal_space, progress_bar, row, scrollable, text,
};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use crate::types::*;
use crate::ui::charts::{self, MultiLineChart};

const SURFACE: Color = Color::from_rgb(0.09, 0.11, 0.16);
const BORDER: Color = Color::from_rgb(0.18, 0.22, 0.30);
const TEXT_PRIMARY: Color = Color::from_rgb(0.8, 0.82, 0.86);
const TEXT_SECONDARY: Color = Color::from_rgb(0.55, 0.58, 0.65);
const ACCENT_CYAN: Color = Color::from_rgb(0.0, 0.74, 0.83);
const ACCENT_GREEN: Color = Color::from_rgb(0.3, 0.74, 0.46);
const ACCENT_ORANGE: Color = Color::from_rgb(1.0, 0.6, 0.0);
const GPU_LINE: Color = Color::from_rgb(1.0, 0.6, 0.0);
const NET_RX: Color = Color::from_rgb(0.13, 0.59, 0.95);
const NET_TX: Color = Color::from_rgb(0.96, 0.27, 0.27);

pub struct Dashboard {
    cpu_chart: MultiLineChart,
    ram_chart: MultiLineChart,
    gpu_chart: MultiLineChart,
    net_chart: MultiLineChart,
}

impl Dashboard {
    pub fn new() -> Self {
        Dashboard {
            cpu_chart: MultiLineChart::new("CPU Usage", 100.0),
            ram_chart: MultiLineChart::new("RAM Usage", 100.0),
            gpu_chart: MultiLineChart::new("GPU Temp", 100.0),
            net_chart: MultiLineChart::new("Network", 0.0),
        }
    }

    pub fn update(&mut self, _metrics: &SystemMetrics, history: &MetricsHistory) {
        self.cpu_chart.set_data(vec![(history.cpu_total.clone(), ACCENT_CYAN)]);
        self.cpu_chart.update();

        self.ram_chart.set_data(vec![(history.ram_usage.clone(), ACCENT_GREEN)]);
        self.ram_chart.update();

        if !history.gpu_temperatures.is_empty() {
            let gpu_data = history.gpu_temperatures[0].clone();
            self.gpu_chart.set_data(vec![(gpu_data, GPU_LINE)]);
            self.gpu_chart.update();
        }

        self.net_chart.set_data(vec![
            (history.net_rx.clone(), NET_RX),
            (history.net_tx.clone(), NET_TX),
        ]);
        self.net_chart.update();
    }

    pub fn view<'a>(&'a self, metrics: &'a SystemMetrics, _history: &'a MetricsHistory) -> Element<'a, DashboardMessage> {
        let content = column![
            cpu_card(metrics, &self.cpu_chart),
            memory_card(metrics, &self.ram_chart),
            gpu_card(metrics, &self.gpu_chart),
            disk_card(metrics),
            network_card(metrics, &self.net_chart),
            battery_card(metrics),
        ]
        .spacing(12)
        .padding(16);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

#[derive(Debug, Clone)]
pub enum DashboardMessage {
    Updated,
}

fn card_style() -> container::Style {
    container::Style::default()
        .background(SURFACE)
        .border(Border::default().color(BORDER).width(1.0).rounded(8.0))
}

fn metric_row<'a>(
    label: &'a str,
    value: String,
    extra: Option<String>,
) -> Element<'a, DashboardMessage> {
    let mut row = row![
        text(label).size(13).color(TEXT_SECONDARY),
        horizontal_space(),
        text(value).size(14).color(TEXT_PRIMARY).font(iced::font::Font::MONOSPACE),
    ];

    if let Some(extra_text) = extra {
        row = row.push(
            text(extra_text)
                .size(12)
                .color(TEXT_SECONDARY)
                .font(iced::font::Font::MONOSPACE),
        );
    }

    row.align_y(Alignment::Center).into()
}

fn chart_canvas<'a>(chart: &'a MultiLineChart) -> Element<'a, DashboardMessage> {
    chart.view().map(|_: charts::Message| DashboardMessage::Updated)
}

fn cpu_card<'a>(metrics: &'a SystemMetrics, chart: &'a MultiLineChart) -> Element<'a, DashboardMessage> {
    let cpu = &metrics.cpu;
    let usage_text = format!("{:.1}%", cpu.total_usage);
    let temp_text = cpu
        .temperature
        .map(|t| format!("{:.0}°C", t))
        .unwrap_or_default();
    let freq_text = cpu
        .frequency
        .map(|f| format!("{} MHz", f))
        .unwrap_or_default();

    let cpu_bar = progress_bar(0.0..=100.0, cpu.total_usage as f32)
        .style(|_: &Theme| progress_bar::Style {
            background: Background::Color(BORDER),
            bar: Background::Color(ACCENT_CYAN),
            border: Border::default().rounded(4),
        });

    container(
        column![
            text("CPU").size(14).color(TEXT_SECONDARY),
            metric_row("Usage", usage_text, Some(format!("{}  {}", temp_text, freq_text))),
            cpu_bar,
            chart_canvas(chart),
        ]
        .spacing(8)
        .padding(16),
    )
    .style(|_: &Theme| card_style())
    .width(Length::Fill)
    .into()
}

fn memory_card<'a>(metrics: &'a SystemMetrics, chart: &'a MultiLineChart) -> Element<'a, DashboardMessage> {
    let mem = &metrics.memory;
    let used_gb = mem.used as f64 / 1024.0 / 1024.0 / 1024.0;
    let total_gb = mem.total as f64 / 1024.0 / 1024.0 / 1024.0;
    let swap_gb = mem.swap_used as f64 / 1024.0 / 1024.0 / 1024.0;

    let mem_bar = progress_bar(0.0..=100.0, mem.percent as f32)
        .style(|_: &Theme| progress_bar::Style {
            background: Background::Color(BORDER),
            bar: Background::Color(ACCENT_GREEN),
            border: Border::default().rounded(4),
        });

    container(
        column![
            text("Memory").size(14).color(TEXT_SECONDARY),
            metric_row("RAM", format!("{:.1}/{:.1} GB", used_gb, total_gb), Some(format!("{:.1}%", mem.percent))),
            mem_bar,
            chart_canvas(chart),
            metric_row("Swap", format!("{:.1} GB", swap_gb), None),
        ]
        .spacing(8)
        .padding(16),
    )
    .style(|_: &Theme| card_style())
    .width(Length::Fill)
    .into()
}

fn gpu_card<'a>(metrics: &'a SystemMetrics, chart: &'a MultiLineChart) -> Element<'a, DashboardMessage> {
    if metrics.gpu.gpus.is_empty() {
        return container(
            column![
                text("GPU").size(14).color(TEXT_SECONDARY),
                text("No GPU detected").size(13).color(TEXT_SECONDARY),
            ]
            .spacing(8)
            .padding(16),
        )
        .style(|_: &Theme| card_style())
        .width(Length::Fill)
        .into();
    }

    let gpu = &metrics.gpu.gpus[0];
    let temp_text = gpu
        .temperature
        .map(|t| format!("{:.0}°C", t))
        .unwrap_or_else(|| "N/A".to_string());
    let usage_text = gpu
        .usage
        .map(|u| format!("{:.1}%", u))
        .unwrap_or_else(|| "N/A".to_string());
    let mem_text = gpu
        .memory_used
        .zip(gpu.memory_total)
        .map(|(used, total)| {
            let used_gb = used as f64 / 1024.0 / 1024.0 / 1024.0;
            let total_gb = total as f64 / 1024.0 / 1024.0 / 1024.0;
            format!("{:.1}/{:.1} GB", used_gb, total_gb)
        })
        .unwrap_or_default();

    let gpu_bar = gpu.usage.map(|u| {
        progress_bar(0.0..=100.0, u as f32).style(|_: &Theme| progress_bar::Style {
            background: Background::Color(BORDER),
            bar: Background::Color(ACCENT_ORANGE),
            border: Border::default().rounded(4),
        })
    });

    let mut col = column![
        text("GPU").size(14).color(TEXT_SECONDARY),
        metric_row(&gpu.name, format!("{}  {}", usage_text, temp_text), Some(mem_text)),
    ]
    .spacing(8);

    if let Some(bar) = gpu_bar {
        col = col.push(bar);
    }
    col = col.push(chart_canvas(chart));

    container(col.padding(16))
        .style(|_: &Theme| card_style())
        .width(Length::Fill)
        .into()
}

fn disk_card<'a>(metrics: &'a SystemMetrics) -> Element<'a, DashboardMessage> {
    let mut col = column![
        text("Disk").size(14).color(TEXT_SECONDARY),
    ]
    .spacing(8);

    for disk in &metrics.disk.disks {
        let used_gb = disk.used as f64 / 1024.0 / 1024.0 / 1024.0;
        let total_gb = disk.total as f64 / 1024.0 / 1024.0 / 1024.0;

        let bar = progress_bar(0.0..=100.0, disk.percent as f32)
            .style(|_: &Theme| progress_bar::Style {
                background: Background::Color(BORDER),
                bar: Background::Color(ACCENT_CYAN),
                border: Border::default().rounded(4),
            });

        col = col.push(metric_row(
            &disk.mount_point,
            format!("{:.1}/{:.1} GB", used_gb, total_gb),
            Some(format!("{:.1}%", disk.percent)),
        ));
        col = col.push(bar);
    }

    container(col.padding(16))
        .style(|_: &Theme| card_style())
        .width(Length::Fill)
        .into()
}

fn network_card<'a>(metrics: &'a SystemMetrics, chart: &'a MultiLineChart) -> Element<'a, DashboardMessage> {
    let rx = metrics.network.total_rx as f64;
    let tx = metrics.network.total_tx as f64;
    let rx_str = if rx > 1_000_000.0 {
        format!("{:.1} MB/s", rx / 1_000_000.0)
    } else if rx > 1_000.0 {
        format!("{:.1} KB/s", rx / 1_000.0)
    } else {
        format!("{:.0} B/s", rx)
    };
    let tx_str = if tx > 1_000_000.0 {
        format!("{:.1} MB/s", tx / 1_000_000.0)
    } else if tx > 1_000.0 {
        format!("{:.1} KB/s", tx / 1_000.0)
    } else {
        format!("{:.0} B/s", tx)
    };

    container(
        column![
            text("Network").size(14).color(TEXT_SECONDARY),
            metric_row("↓ RX / ↑ TX", rx_str, Some(tx_str)),
            chart_canvas(chart),
        ]
        .spacing(8)
        .padding(16),
    )
    .style(|_: &Theme| card_style())
    .width(Length::Fill)
    .into()
}

fn battery_card<'a>(metrics: &'a SystemMetrics) -> Element<'a, DashboardMessage> {
    if let Some(cap) = metrics.battery.capacity {
        let status = if metrics.battery.charging.unwrap_or(false) {
            "Charging"
        } else {
            "Discharging"
        };
        let bar = progress_bar(0.0..=100.0, cap as f32).style(move |_: &Theme| {
            progress_bar::Style {
                background: Background::Color(BORDER),
                bar: Background::Color(if cap > 50.0 { ACCENT_GREEN } else { ACCENT_ORANGE }),
                border: Border::default().rounded(4),
            }
        });

        container(
            column![
                text("Battery").size(14).color(TEXT_SECONDARY),
                metric_row(status, format!("{:.0}%", cap), None),
                bar,
            ]
            .spacing(8)
            .padding(16),
        )
        .style(|_: &Theme| card_style())
        .width(Length::Fill)
        .into()
    } else {
        container(
            column![
                text("Battery").size(14).color(TEXT_SECONDARY),
                text("No battery detected").size(13).color(TEXT_SECONDARY),
            ]
            .spacing(8)
            .padding(16),
        )
        .style(|_: &Theme| card_style())
        .width(Length::Fill)
        .into()
    }
}
