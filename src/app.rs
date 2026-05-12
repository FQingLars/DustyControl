use std::sync::mpsc;
use std::time::Duration;

use iced::widget::{button, column, container, horizontal_space, row, text, Column};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Subscription, Task, Theme};

use crate::alert::AlertEngine;
use crate::audio::AudioEngine;
use crate::config::Config;
use crate::monitor::{start_monitoring, MonitorCommand};
use crate::types::*;
use crate::ui::dashboard::Dashboard;
use crate::ui::processes::{ProcessMessage, ProcessTable};
use crate::ui::settings::{SettingsMessage, SettingsPane};

const BG_DARK: Color = Color::from_rgb(0.05, 0.06, 0.09);
const TEXT_PRIMARY: Color = Color::from_rgb(0.8, 0.82, 0.86);
const TEXT_SECONDARY: Color = Color::from_rgb(0.55, 0.58, 0.65);
const ACCENT_CYAN: Color = Color::from_rgb(0.0, 0.74, 0.83);

#[derive(Debug, Clone)]
pub enum Message {
    Tick,
    DashboardMsg,
    Process(ProcessMessage),
    TabChanged(Tab),
    Settings(SettingsMessage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Processes,
    Alerts,
    Settings,
}

pub struct App {
    config: Config,
    audio: AudioEngine,
    alert_engine: AlertEngine,
    monitor_tx: mpsc::Sender<MonitorCommand>,
    metrics_rx: mpsc::Receiver<SystemMetrics>,
    current_metrics: Option<SystemMetrics>,
    history: MetricsHistory,
    dashboard: Dashboard,
    process_table: ProcessTable,
    settings_pane: SettingsPane,
    active_tab: Tab,
    triggered_alerts: Vec<AlertEventInfo>,
}

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let config = Config::load();
        let audio = AudioEngine::new();
        let alert_engine = AlertEngine::new(&config);

        let (monitor_tx, monitor_rx) = mpsc::channel();
        let (metrics_tx, metrics_rx) = mpsc::channel();
        start_monitoring(monitor_rx, metrics_tx);

        let history = MetricsHistory::new(4, 1);
        let settings_pane = SettingsPane::new(&config);

        let app = App {
            config,
            audio,
            alert_engine,
            monitor_tx,
            metrics_rx,
            current_metrics: None,
            history,
            dashboard: Dashboard::new(),
            process_table: ProcessTable::new(),
            settings_pane,
            active_tab: Tab::Dashboard,
            triggered_alerts: Vec::new(),
        };

        (app, Task::none())
    }

    pub fn start() -> iced::Result {
        iced::application("DustyControl", update, view)
            .subscription(subscription)
            .theme(theme)
            .window_size(iced::Size::new(1200.0, 800.0))
            .run_with(App::new)
    }
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Tick => {
            while let Ok(metrics) = app.metrics_rx.try_recv() {
                let alerts = app.alert_engine.check(&metrics, &app.audio);
                if !alerts.is_empty() {
                    app.triggered_alerts.extend(alerts);
                    if app.triggered_alerts.len() > 100 {
                        app.triggered_alerts.drain(0..app.triggered_alerts.len() - 100);
                    }
                }
                app.history.push(&metrics);
                app.dashboard.update(&metrics, &app.history);
                app.current_metrics = Some(metrics);
            }
            Task::none()
        }
        Message::DashboardMsg => Task::none(),
        Message::Process(action) => match action {
            ProcessMessage::Kill(pid) => {
                let _ = std::process::Command::new("kill")
                    .arg(pid.to_string())
                    .spawn();
                Task::none()
            }
            ProcessMessage::Renice(pid, nice) => {
                let _ = std::process::Command::new("renice")
                    .arg(nice.to_string())
                    .arg("-p")
                    .arg(pid.to_string())
                    .spawn();
                Task::none()
            }
            ProcessMessage::Select(_) => Task::none(),
        },
        Message::TabChanged(tab) => {
            if tab == Tab::Settings {
                app.settings_pane.sync_from_config(&app.config);
            }
            app.active_tab = tab;
            Task::none()
        }
        Message::Settings(msg) => {
            let needs_alert_reload = matches!(msg, SettingsMessage::Save);
            app.settings_pane.update(msg, &mut app.config);
            if needs_alert_reload {
                app.alert_engine = crate::alert::AlertEngine::new(&app.config);
            }
            Task::none()
        }
    }
}

fn view<'a>(app: &'a App) -> Element<'a, Message> {
    let header = header_view(app);
    let content = content_view(app);
    let status_bar = status_bar_view(app);

    container(
        column![header, content, status_bar]
            .spacing(0)
            .height(Length::Fill),
    )
    .style(|_: &Theme| container::Style::default().background(BG_DARK))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn subscription(app: &App) -> Subscription<Message> {
    iced::time::every(Duration::from_millis(app.config.settings.update_interval_ms))
        .map(|_| Message::Tick)
}

fn theme(_app: &App) -> Theme {
    Theme::Dark
}

fn header_view(app: &App) -> Element<'_, Message> {
    let title = text("DustyControl").size(18).color(ACCENT_CYAN);

    let tabs = row![
        tab_button("Dashboard", Tab::Dashboard, &app.active_tab),
        tab_button("Processes", Tab::Processes, &app.active_tab),
        tab_button("Alerts", Tab::Alerts, &app.active_tab),
        tab_button("Settings", Tab::Settings, &app.active_tab),
    ]
    .spacing(4);

    row![
        title,
        horizontal_space(),
        tabs,
        horizontal_space(),
        text(format!("v{}", env!("CARGO_PKG_VERSION")))
            .size(11)
            .color(TEXT_SECONDARY),
    ]
    .align_y(Alignment::Center)
    .padding(12)
    .spacing(16)
    .into()
}

fn tab_button<'a>(label: &'a str, tab: Tab, active: &'a Tab) -> Element<'a, Message> {
    let is_active = *active == tab;
    let tab_for_style = tab.clone();
    let active_color = ACCENT_CYAN;
    let inactive_color = TEXT_SECONDARY;

    button(text(label).size(13).color(if is_active { active_color } else { inactive_color }))
        .on_press(Message::TabChanged(tab))
        .style(move |_: &Theme, _: button::Status| {
            if *active == tab_for_style {
                button::Style {
                    background: Some(Background::Color(Color::from_rgba(0.0, 0.74, 0.83, 0.1))),
                    text_color: active_color,
                    border: Border::default().color(ACCENT_CYAN).width(1).rounded(4),
                    shadow: Shadow::default(),
                }
            } else {
                button::Style {
                    background: None,
                    text_color: inactive_color,
                    border: Border::default(),
                    shadow: Shadow::default(),
                }
            }
        })
        .padding(iced::Padding::from([6, 12]))
        .into()
}

fn content_view(app: &App) -> Element<'_, Message> {
    match app.active_tab {
        Tab::Dashboard => {
            if let Some(ref metrics) = app.current_metrics {
                app.dashboard
                    .view(metrics, &app.history)
                    .map(|_| Message::DashboardMsg)
            } else {
                container(
                    text("Waiting for metrics...")
                        .size(16)
                        .color(TEXT_SECONDARY),
                )
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
            }
        }
        Tab::Processes => {
            if let Some(ref metrics) = app.current_metrics {
                app.process_table
                    .view(&metrics.processes)
                    .map(Message::Process)
            } else {
                container(text("Loading...").size(16).color(TEXT_SECONDARY))
                    .center_x(Length::Fill)
                    .center_y(Length::Fill)
                    .into()
            }
        }
        Tab::Alerts => alerts_view(app),
        Tab::Settings => app.settings_pane.view(&app.config).map(Message::Settings),
    }
}

fn alerts_view(app: &App) -> Element<'_, Message> {
    if app.triggered_alerts.is_empty() {
        return container(
            text("No alerts triggered").size(16).color(TEXT_SECONDARY),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
    }

    let mut col = Column::new().spacing(4).padding(16);

    for alert in app.triggered_alerts.iter().rev() {
        let event_text = alert.event.to_string();
        let value_text = format!("{:.1} / {:.1}", alert.current_value, alert.threshold);
        let sound_text = format!("🔊 {}", alert.sound);

        let alert_row = row![
            text(event_text).size(13).color(ACCENT_CYAN).width(Length::Fixed(150.0)),
            text(value_text)
                .size(13)
                .color(TEXT_PRIMARY)
                .font(iced::font::Font::MONOSPACE)
                .width(Length::Fixed(120.0)),
            text(sound_text).size(13).color(TEXT_SECONDARY),
        ]
        .align_y(Alignment::Center)
        .spacing(8)
        .padding(8);

        let alert_container = container(alert_row).style(|_: &Theme| {
            container::Style::default()
                .background(Color::from_rgba(0.96, 0.27, 0.27, 0.08))
                .border(
                    Border::default()
                        .color(Color::from_rgba(0.96, 0.27, 0.27, 0.3))
                        .width(1)
                        .rounded(4),
                )
        });

        col = col.push(alert_container);
    }

    container(col)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn status_bar_view(app: &App) -> Element<'_, Message> {
    let info = if let Some(ref metrics) = app.current_metrics {
        format!(
            "CPU: {:.1}% | RAM: {:.1}% | Processes: {}",
            metrics.cpu.total_usage,
            metrics.memory.percent,
            metrics.processes.len(),
        )
    } else {
        "Initializing...".to_string()
    };

    container(
        text(info)
            .size(11)
            .color(TEXT_SECONDARY)
            .font(iced::font::Font::MONOSPACE),
    )
    .style(|_: &Theme| {
        container::Style::default()
            .background(Color::from_rgb(0.04, 0.05, 0.08))
            .border(
                Border::default()
                    .color(Color::from_rgb(0.12, 0.14, 0.20))
                    .width(1)
                    .rounded(0),
            )
    })
    .padding(iced::Padding::from([6, 12]))
    .width(Length::Fill)
    .into()
}
