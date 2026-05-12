use iced::widget::{button, column, container, row, scrollable, text, Column};
use iced::{Alignment, Color, Element, Length, Theme};

use crate::types::ProcessInfo;

const TEXT_PRIMARY: Color = Color::from_rgb(0.8, 0.82, 0.86);
const TEXT_SECONDARY: Color = Color::from_rgb(0.55, 0.58, 0.65);
const HEADER_BG: Color = Color::from_rgb(0.12, 0.14, 0.20);
const ROW_HOVER: Color = Color::from_rgb(0.11, 0.13, 0.19);
const ACCENT_CYAN: Color = Color::from_rgb(0.0, 0.74, 0.83);
const ACCENT_RED: Color = Color::from_rgb(0.96, 0.27, 0.27);
const ACCENT_GREEN: Color = Color::from_rgb(0.3, 0.74, 0.46);

#[derive(Clone, Debug)]
pub enum ProcessMessage {
    Kill(u32),
    Renice(u32, i32),
    Select(u32),
}

pub struct ProcessTable {
    selected_pid: Option<u32>,
}

impl ProcessTable {
    pub fn new() -> Self {
        ProcessTable { selected_pid: None }
    }

    pub fn view<'a>(&self, processes: &'a [ProcessInfo]) -> Element<'a, ProcessMessage> {
        let header = row![
            text("PID").size(12).color(TEXT_SECONDARY).width(70),
            text("Name").size(12).color(TEXT_SECONDARY).width(Length::Fill),
            text("CPU%").size(12).color(TEXT_SECONDARY).width(60),
            text("MEM").size(12).color(TEXT_SECONDARY).width(70),
            text("State").size(12).color(TEXT_SECONDARY).width(70),
            text("Actions").size(12).color(TEXT_SECONDARY).width(120),
        ]
        .align_y(Alignment::Center)
        .padding(8)
        .spacing(8);

        let header_container = container(header)
            .style(|_: &Theme| container::Style::default().background(HEADER_BG))
            .width(Length::Fill);

        let mut rows = Column::new().spacing(1);

        for proc in processes.iter().take(100) {
            let mem_mb = proc.memory as f64 / 1024.0;
            let mem_str = if mem_mb > 1024.0 {
                format!("{:.1}G", mem_mb / 1024.0)
            } else {
                format!("{:.0}M", mem_mb)
            };

            let is_selected = self.selected_pid == Some(proc.pid);

            let kill_btn = button(text("KILL")
                .size(10)
                .color(if is_selected { ACCENT_RED } else { TEXT_SECONDARY }))
                .on_press(ProcessMessage::Kill(proc.pid))
                .style(flat_button_style(if is_selected { ACCENT_RED } else { TEXT_SECONDARY }))
                .padding(iced::Padding::from([2, 6]));

            let reniced_btn = button(text("+5")
                .size(10)
                .color(if is_selected { ACCENT_GREEN } else { TEXT_SECONDARY }))
                .on_press(ProcessMessage::Renice(proc.pid, 5))
                .style(flat_button_style(if is_selected { ACCENT_GREEN } else { TEXT_SECONDARY }))
                .padding(iced::Padding::from([2, 6]));

            let procd_btn = button(text("-5")
                .size(10)
                .color(if is_selected { ACCENT_GREEN } else { TEXT_SECONDARY }))
                .on_press(ProcessMessage::Renice(proc.pid, -5))
                .style(flat_button_style(if is_selected { ACCENT_GREEN } else { TEXT_SECONDARY }))
                .padding(iced::Padding::from([2, 6]));

            let proc_row = row![
                text(proc.pid.to_string())
                    .size(12)
                    .color(TEXT_SECONDARY)
                    .font(iced::font::Font::MONOSPACE)
                    .width(70),
                text(&proc.name)
                    .size(12)
                    .color(TEXT_PRIMARY)
                    .width(Length::Fill),
                text(format!("{:.1}", proc.cpu_usage))
                    .size(12)
                    .color(ACCENT_CYAN)
                    .font(iced::font::Font::MONOSPACE)
                    .width(60),
                text(mem_str)
                    .size(12)
                    .color(TEXT_PRIMARY)
                    .font(iced::font::Font::MONOSPACE)
                    .width(70),
                text(&proc.state)
                    .size(12)
                    .color(TEXT_SECONDARY)
                    .width(70),
                row![kill_btn, reniced_btn, procd_btn]
                    .spacing(4)
                    .width(120),
            ]
            .align_y(Alignment::Center)
            .padding(6)
            .spacing(8);

            let row_style = if is_selected {
                |_: &Theme| container::Style::default().background(ROW_HOVER)
            } else {
                |_: &Theme| container::Style::default()
            };

            let row_container = container(proc_row)
                .style(row_style)
                .width(Length::Fill);

            rows = rows.push(row_container);
        }

        let content = column![header_container, rows].spacing(2);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn flat_button_style(color: Color) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_: &Theme, status: button::Status| {
        let bg = match status {
            button::Status::Hovered => Some(iced::Background::Color(Color::from_rgba(
                color.r,
                color.g,
                color.b,
                0.15,
            ))),
            _ => None,
        };
        button::Style {
            background: bg,
            text_color: color,
            border: iced::Border::default()
                .color(Color::from_rgba(color.r, color.g, color.b, 0.3))
                .width(1)
                .rounded(3),
            shadow: iced::Shadow::default(),
        }
    }
}
