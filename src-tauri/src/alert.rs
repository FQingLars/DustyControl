use std::collections::HashMap;

use crate::config::Config;
use crate::types::*;

pub struct AlertEngine {
    rules: Vec<AlertRule>,
    cooldowns: HashMap<String, std::time::Instant>,
    cooldown_duration: std::time::Duration,
}

impl AlertEngine {
    pub fn new(config: &Config) -> Self {
        AlertEngine {
            rules: config.alerts.events.clone(),
            cooldowns: HashMap::new(),
            cooldown_duration: std::time::Duration::from_secs(10),
        }
    }

    pub fn check(
        &mut self,
        metrics: &SystemMetrics,
    ) -> Vec<AlertEventInfo> {
        if self.rules.is_empty() {
            return Vec::new();
        }

        let mut triggered = Vec::new();

        for rule in &self.rules {
            let value = self.get_value(rule.event.clone(), metrics);
            if let Some(val) = value {
                if val >= rule.threshold {
                    let cooldown_key = format!("{}_{}", rule.event, rule.threshold);
                    let should_fire = self
                        .cooldowns
                        .get(&cooldown_key)
                        .map(|t| t.elapsed() >= self.cooldown_duration)
                        .unwrap_or(true);

                    if should_fire {
                        self.cooldowns
                            .insert(cooldown_key, std::time::Instant::now());
                    }

                    triggered.push(AlertEventInfo {
                        event: rule.event.clone(),
                        current_value: val,
                        threshold: rule.threshold,
                        sound: rule.sound.clone(),
                    });
                }
            }
        }

        triggered
    }

    fn get_value(&self, event: AlertEvent, metrics: &SystemMetrics) -> Option<f64> {
        match event {
            AlertEvent::CpuTemperature => metrics.cpu.temperature,
            AlertEvent::GpuTemperature => {
                metrics
                    .gpu
                    .gpus
                    .first()
                    .and_then(|g| g.temperature)
            }
            AlertEvent::CpuUsage => Some(metrics.cpu.total_usage),
            AlertEvent::RamUsage => Some(metrics.memory.percent),
            AlertEvent::DiskUsage => {
                metrics
                    .disk
                    .disks
                    .first()
                    .map(|d| d.percent)
            }
            AlertEvent::NetworkUsage => Some(
                (metrics.network.total_rx + metrics.network.total_tx) as f64,
            ),
            AlertEvent::BatteryLevel => metrics.battery.capacity,
            AlertEvent::ProcessCount => Some(metrics.processes.len() as f64),
        }
    }
}
