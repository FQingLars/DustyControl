use iced::widget::{button, column, container, pick_list, row, scrollable, text, text_input, toggler};
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

use crate::config::Config;
use crate::types::{AlertEvent, AlertRule};

const SURFACE: Color = Color::from_rgb(0.09, 0.11, 0.16);
const SURFACE_DARK: Color = Color::from_rgb(0.06, 0.08, 0.12);
const BORDER: Color = Color::from_rgb(0.18, 0.22, 0.30);
const TEXT_PRIMARY: Color = Color::from_rgb(0.8, 0.82, 0.86);
const TEXT_SECONDARY: Color = Color::from_rgb(0.55, 0.58, 0.65);
const ACCENT_CYAN: Color = Color::from_rgb(0.0, 0.74, 0.83);
const ACCENT_GREEN: Color = Color::from_rgb(0.26, 0.78, 0.41);
const ACCENT_RED: Color = Color::from_rgb(0.96, 0.27, 0.27);

const SOUND_PRESETS: &[&str] = &[
    "beep", "alert", "alarm", "notification", "critical", "warnsiren", "chime",
];

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    IntervalEdited(String),
    AlertsToggled(bool),
    NewRuleEventChanged(AlertEvent),
    NewRuleThresholdEdited(String),
    NewRuleSoundChanged(String),
    AddRule,
    DeleteRule(usize),
    Save,
}

pub struct SettingsPane {
    interval_input: String,
    alerts_enabled: bool,
    new_rule_event: AlertEvent,
    new_rule_threshold: String,
    new_rule_sound: String,
    save_feedback: Option<String>,
    event_options: Vec<AlertEvent>,
    sound_options: Vec<String>,
}

impl SettingsPane {
    pub fn new(config: &Config) -> Self {
        Self {
            interval_input: config.settings.update_interval_ms.to_string(),
            alerts_enabled: config.alerts.enabled,
            new_rule_event: AlertEvent::CpuTemperature,
            new_rule_threshold: String::new(),
            new_rule_sound: "beep".to_string(),
            save_feedback: None,
            event_options: vec![
                AlertEvent::CpuTemperature,
                AlertEvent::GpuTemperature,
                AlertEvent::CpuUsage,
                AlertEvent::RamUsage,
                AlertEvent::DiskUsage,
                AlertEvent::NetworkUsage,
                AlertEvent::BatteryLevel,
                AlertEvent::ProcessCount,
            ],
            sound_options: SOUND_PRESETS.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn sync_from_config(&mut self, config: &Config) {
        self.interval_input = config.settings.update_interval_ms.to_string();
        self.alerts_enabled = config.alerts.enabled;
    }

    pub fn update(&mut self, msg: SettingsMessage, config: &mut Config) {
        match msg {
            SettingsMessage::IntervalEdited(v) => {
                self.interval_input = v;
                self.save_feedback = None;
            }
            SettingsMessage::AlertsToggled(v) => {
                self.alerts_enabled = v;
                self.save_feedback = None;
            }
            SettingsMessage::NewRuleEventChanged(v) => {
                self.new_rule_event = v;
            }
            SettingsMessage::NewRuleThresholdEdited(v) => {
                self.new_rule_threshold = v;
            }
            SettingsMessage::NewRuleSoundChanged(v) => {
                self.new_rule_sound = v;
            }
            SettingsMessage::AddRule => {
                if let Ok(threshold) = self.new_rule_threshold.trim().parse::<f64>() {
                    config.alerts.events.push(AlertRule {
                        event: self.new_rule_event.clone(),
                        threshold,
                        sound: self.new_rule_sound.clone(),
                    });
                    self.new_rule_threshold.clear();
                    self.new_rule_sound = "beep".to_string();
                    self.save_feedback = None;
                }
            }
            SettingsMessage::DeleteRule(idx) => {
                if idx < config.alerts.events.len() {
                    config.alerts.events.remove(idx);
                    self.save_feedback = None;
                }
            }
            SettingsMessage::Save => {
                if let Ok(ms) = self.interval_input.parse::<u64>() {
                    config.settings.update_interval_ms = ms;
                }
                config.alerts.enabled = self.alerts_enabled;
                match config.save() {
                    Ok(()) => {
                        self.save_feedback =
                            Some("Configuration saved successfully.".to_string());
                    }
                    Err(e) => {
                        self.save_feedback = Some(format!("Save failed: {e}"));
                    }
                }
            }
        }
    }

    pub fn view<'a>(&'a self, config: &'a Config) -> Element<'a, SettingsMessage> {
        let general_card = container(
            column![
                section_title("General Settings"),
                row![
                    text("Update Interval")
                        .size(13)
                        .color(TEXT_SECONDARY)
                        .width(Length::Fixed(200.0)),
                    text_input("1000", &self.interval_input)
                        .on_input(SettingsMessage::IntervalEdited)
                        .width(Length::Fixed(120.0))
                        .style(|_: &Theme, _: text_input::Status| text_input::Style {
                            background: Background::Color(SURFACE_DARK),
                            border: Border::default()
                                .color(BORDER)
                                .width(1)
                                .rounded(4),
                            icon: TEXT_SECONDARY,
                            placeholder: TEXT_SECONDARY,
                            value: TEXT_PRIMARY,
                            selection: ACCENT_CYAN,
                        }),
                    text("ms").size(13).color(TEXT_SECONDARY),
                ]
                .align_y(Alignment::Center)
                .spacing(8)
                .padding(8),
                row![
                    text("Alerts")
                        .size(13)
                        .color(TEXT_SECONDARY)
                        .width(Length::Fixed(200.0)),
                    toggler(self.alerts_enabled)
                        .on_toggle(SettingsMessage::AlertsToggled),
                    text(if self.alerts_enabled { "Enabled" } else { "Disabled" })
                        .size(13)
                        .color(if self.alerts_enabled {
                            ACCENT_GREEN
                        } else {
                            TEXT_SECONDARY
                        }),
                ]
                .align_y(Alignment::Center)
                .padding(8),
            ]
            .spacing(8),
        )
        .style(card_style)
        .width(Length::Fill)
        .padding(16);

        let mut rules_list = column![].spacing(4);

        for (i, rule) in config.alerts.events.iter().enumerate() {
            let rule_row = row![
                text(rule.event.to_string())
                    .size(13)
                    .color(ACCENT_CYAN)
                    .width(Length::Fixed(160.0)),
                text(format!("> {}", rule.threshold))
                    .size(13)
                    .color(TEXT_PRIMARY)
                    .font(iced::font::Font::MONOSPACE)
                    .width(Length::Fixed(80.0)),
                text(&rule.sound)
                    .size(13)
                    .color(TEXT_SECONDARY)
                    .width(Length::Fill),
                button(text("\u{2715}").size(12))
                    .on_press(SettingsMessage::DeleteRule(i))
                    .style(move |_: &Theme, status: button::Status| {
                        let hovered = matches!(status, button::Status::Hovered);
                        button::Style {
                            background: if hovered {
                                Some(Background::Color(Color::from_rgba(
                                    0.96, 0.27, 0.27, 0.2,
                                )))
                            } else {
                                None
                            },
                            text_color: if hovered { ACCENT_RED } else { TEXT_SECONDARY },
                            border: Border::default()
                                .color(if hovered { ACCENT_RED } else { BORDER })
                                .width(1)
                                .rounded(4),
                            shadow: Shadow::default(),
                        }
                    })
                    .padding(4),
            ]
            .align_y(Alignment::Center)
            .spacing(8)
            .padding(8);

            rules_list = rules_list.push(container(rule_row).style(card_style).width(Length::Fill));
        }

        let new_rule_section = container(
            column![
                row![
                    pick_list(
                        self.event_options.clone(),
                        Some(self.new_rule_event.clone()),
                        SettingsMessage::NewRuleEventChanged,
                    )
                    .width(Length::Fixed(160.0)),
                    text_input("threshold", &self.new_rule_threshold)
                        .on_input(SettingsMessage::NewRuleThresholdEdited)
                        .width(Length::Fixed(80.0))
                        .style(|_: &Theme, _: text_input::Status| text_input::Style {
                            background: Background::Color(SURFACE_DARK),
                            border: Border::default()
                                .color(BORDER)
                                .width(1)
                                .rounded(4),
                            icon: TEXT_SECONDARY,
                            placeholder: TEXT_SECONDARY,
                            value: TEXT_PRIMARY,
                            selection: ACCENT_CYAN,
                        }),
                    pick_list(
                        self.sound_options.clone(),
                        Some(self.new_rule_sound.clone()),
                        SettingsMessage::NewRuleSoundChanged,
                    )
                    .width(Length::Fixed(120.0)),
                    button(text("+").size(16))
                        .on_press(SettingsMessage::AddRule)
                        .style(|_: &Theme, status: button::Status| {
                            let hovered = matches!(status, button::Status::Hovered);
                            button::Style {
                                background: Some(Background::Color(Color::from_rgba(
                                    0.26,
                                    0.78,
                                    0.41,
                                    if hovered { 0.2 } else { 0.1 },
                                ))),
                                text_color: ACCENT_GREEN,
                                border: Border::default()
                                    .color(ACCENT_GREEN)
                                    .width(1)
                                    .rounded(4),
                                shadow: Shadow::default(),
                            }
                        })
                        .padding(iced::Padding::from([4, 12])),
                ]
                .align_y(Alignment::Center)
                .spacing(8)
                .padding(8),
            ]
            .spacing(8),
        )
        .style(card_style)
        .width(Length::Fill)
        .padding(16);

        let rules_card = container(
            column![
                section_title("Alert Rules"),
                new_rule_section,
                rules_list,
            ]
            .spacing(8),
        )
        .style(card_style)
        .width(Length::Fill)
        .padding(16);

        let mut save_row = row![
            button(text("Save Configuration").size(14))
                .on_press(SettingsMessage::Save)
                .style(|_: &Theme, status: button::Status| {
                    let hovered = matches!(status, button::Status::Hovered);
                    button::Style {
                        background: Some(Background::Color(Color::from_rgba(
                            0.0,
                            0.74,
                            0.83,
                            if hovered { 0.2 } else { 0.1 },
                        ))),
                        text_color: ACCENT_CYAN,
                        border: Border::default()
                            .color(ACCENT_CYAN)
                            .width(1)
                            .rounded(4),
                        shadow: Shadow::default(),
                    }
                })
                .padding(iced::Padding::from([8, 20])),
        ]
        .align_y(Alignment::Center)
        .spacing(16);

        if let Some(ref feedback) = self.save_feedback {
            let color = if feedback.starts_with("Save failed") {
                ACCENT_RED
            } else {
                ACCENT_GREEN
            };
            save_row =
                save_row.push(text(feedback).size(12).color(color));
        }

        let content = column![general_card, rules_card, save_row,]
            .spacing(16)
            .padding(16);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn card_style(_theme: &Theme) -> container::Style {
    container::Style::default()
        .background(SURFACE)
        .border(Border::default().color(BORDER).width(1).rounded(4))
}

fn section_title<'a>(label: &'a str) -> Element<'a, SettingsMessage> {
    text(label).size(16).color(TEXT_PRIMARY).into()
}
