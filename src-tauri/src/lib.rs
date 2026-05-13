mod alert;
mod config;
mod monitor;
mod types;

use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use alert::AlertEngine;
use config::Config;
use monitor::Monitor;
use types::*;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn get_config() -> Config {
    Config::load()
}

#[tauri::command]
fn save_config(cfg: Config) -> Result<(), String> {
    cfg.save().map_err(|e| e.to_string())
}

#[tauri::command]
fn kill_process(pid: u32) -> Result<(), String> {
    let system = sysinfo::System::new_all();
    if let Some(process) = system.process(sysinfo::Pid::from_u32(pid)) {
        process.kill();
        Ok(())
    } else {
        Err(format!("Process {} not found", pid))
    }
}

#[tauri::command]
fn renice_process(pid: u32, delta: i32) -> Result<(), String> {
    let current = unsafe { libc::getpriority(libc::PRIO_PROCESS, pid) };
    let new_nice = (current as i32 + delta).clamp(-20, 19);
    let result = unsafe { libc::setpriority(libc::PRIO_PROCESS, pid, new_nice) };
    if result == 0 {
        Ok(())
    } else {
        Err(format!("Failed to renice process {}", pid))
    }
}

#[derive(Clone, serde::Serialize)]
struct StatusPayload {
    cpu: f64,
    ram: f64,
    process_count: usize,
    cpu_str: String,
    ram_str: String,
}

fn run_monitoring_loop(app: AppHandle) {
    thread::spawn(move || {
        let config = Config::load();
        let mut monitor = Monitor::new();
        let mut alert_engine = AlertEngine::new(&config);
        let cpu_count = monitor.cpu_core_count();
        let gpu_count = monitor.gpu_count();
        let mut history = MetricsHistory::new(cpu_count, gpu_count);
        let interval = Duration::from_millis(config.settings.update_interval_ms);

        loop {
            let metrics = monitor.collect();

            let alerts = alert_engine.check(&metrics);

            history.push(&metrics);

            let payload = MetricsUpdatePayload {
                metrics: metrics.clone(),
                history: history.snapshot(),
            };
            let _ = app.emit("metrics-update", &payload);

            if !alerts.is_empty() {
                for alert in &alerts {
                    let _ = app.emit("alert-triggered", alert);
                }
            }

            let total_gb = metrics.memory.total as f64 / 1_000_000_000.0;
            let used_gb = metrics.memory.used as f64 / 1_000_000_000.0;
            let status = StatusPayload {
                cpu: metrics.cpu.total_usage,
                ram: metrics.memory.percent,
                process_count: metrics.processes.len(),
                cpu_str: format!("{:.1}%", metrics.cpu.total_usage),
                ram_str: format!("{:.1}/{:.1} GB", used_gb, total_gb),
            };
            let _ = app.emit("status-update", &status);

            thread::sleep(interval);
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_config,
            save_config,
            kill_process,
            renice_process,
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            run_monitoring_loop(handle);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
