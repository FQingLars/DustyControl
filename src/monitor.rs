use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use sysinfo::System;

use crate::types::*;

pub struct Monitor {
    sys: System,
    nvidia_gpu: Option<NvidiaGpuMonitor>,
    sysfs_gpu_monitors: Vec<SysfsGpuMonitor>,
    prev_net_rx: u64,
    prev_net_tx: u64,
    prev_time: Instant,
}

pub fn start_monitoring(rx: mpsc::Receiver<MonitorCommand>, tx: mpsc::Sender<SystemMetrics>) {
    thread::spawn(move || {
        let mut monitor = Monitor::new();
        let mut running = true;

        while running {
            let deadline = Instant::now() + Duration::from_millis(UPDATE_INTERVAL_MS);

            let metrics = monitor.collect();
            let _ = tx.send(metrics);

            while let Ok(cmd) = rx.try_recv() {
                match cmd {
                    MonitorCommand::Shutdown => running = false,
                    MonitorCommand::SetInterval(_) => {}
                }
            }

            let elapsed = Instant::now().duration_since(deadline);
            if elapsed < Duration::from_millis(UPDATE_INTERVAL_MS) {
                thread::sleep(Duration::from_millis(UPDATE_INTERVAL_MS) - elapsed);
            }
        }
    });
}

pub enum MonitorCommand {
    Shutdown,
    SetInterval(u64),
}

impl Monitor {
    pub fn new() -> Self {
        let sys = System::new_all();

        let nvidia_gpu = NvidiaGpuMonitor::new();

        let mut sysfs_gpu_monitors = Vec::new();
        sysfs_gpu_monitors.push(SysfsGpuMonitor::new("amdgpu"));
        sysfs_gpu_monitors.push(SysfsGpuMonitor::new("i915"));

        Monitor {
            sys,
            nvidia_gpu,
            sysfs_gpu_monitors,
            prev_net_rx: 0,
            prev_net_tx: 0,
            prev_time: Instant::now(),
        }
    }

    pub fn collect(&mut self) -> SystemMetrics {
        self.sys.refresh_all();

        let cpu = self.collect_cpu();
        let memory = self.collect_memory();
        let disk = collect_disk_procfs();
        let network = self.collect_network();
        let gpu = self.collect_gpu();
        let battery = collect_battery_sysfs();
        let processes = self.collect_processes();

        SystemMetrics {
            cpu,
            memory,
            disk,
            network,
            gpu,
            processes,
            battery,
        }
    }

    fn collect_cpu(&self) -> CpuMetrics {
        let total_usage = self.sys.global_cpu_usage() as f64;
        let per_core_usage: Vec<f64> = self
            .sys
            .cpus()
            .iter()
            .map(|c| c.cpu_usage() as f64)
            .collect();

        let temperature = read_cpu_temp_sysfs();
        let frequency = self.sys.cpus().first().map(|c| c.frequency());

        let load = sysinfo::System::load_average();
        let load_avg = [load.one, load.five, load.fifteen];

        CpuMetrics {
            total_usage,
            per_core_usage,
            temperature,
            frequency,
            load_avg,
        }
    }

    fn collect_memory(&self) -> MemoryMetrics {
        let total = self.sys.total_memory();
        let used = self.sys.used_memory();
        let available = self.sys.available_memory();
        let cached = total.saturating_sub(available).saturating_sub(used);
        let swap_total = self.sys.total_swap();
        let swap_used = self.sys.used_swap();
        let percent = if total > 0 {
            (used as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        MemoryMetrics {
            total,
            used,
            cached,
            available,
            swap_total,
            swap_used,
            percent,
        }
    }

    fn collect_network(&mut self) -> NetworkMetrics {
        let now = Instant::now();
        let dt = now.duration_since(self.prev_time).as_secs_f64();

        let (rx_bytes, tx_bytes) = read_net_procfs();
        let rx_rate = if dt > 0.0 {
            (rx_bytes.saturating_sub(self.prev_net_rx)) as f64 / dt
        } else {
            0.0
        };
        let tx_rate = if dt > 0.0 {
            (tx_bytes.saturating_sub(self.prev_net_tx)) as f64 / dt
        } else {
            0.0
        };

        let interfaces = read_net_interfaces_procfs();

        self.prev_net_rx = rx_bytes;
        self.prev_net_tx = tx_bytes;
        self.prev_time = now;

        NetworkMetrics {
            interfaces,
            total_rx: rx_rate as u64,
            total_tx: tx_rate as u64,
        }
    }

    fn collect_gpu(&mut self) -> GpuMetrics {
        let mut gpus = Vec::new();

        if let Some(ref mut nv) = self.nvidia_gpu {
            gpus.extend(nv.collect());
        }

        for monitor in &self.sysfs_gpu_monitors {
            gpus.extend(monitor.collect());
        }

        GpuMetrics { gpus }
    }

    fn collect_processes(&self) -> Vec<ProcessInfo> {
        let total_mem = self.sys.total_memory();

        let mut procs: Vec<ProcessInfo> = self
            .sys
            .processes()
            .iter()
            .map(|(pid, process)| {
                let mem = process.memory();
                let mem_pct = if total_mem > 0 {
                    (mem as f64 / total_mem as f64 * 100.0) as f32
                } else {
                    0.0
                };

                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    cpu_usage: process.cpu_usage(),
                    memory: mem,
                    memory_percent: mem_pct,
                    state: format!("{:?}", process.status()),
                }
            })
            .collect();

        procs.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        procs.truncate(200);
        procs
    }
}

fn read_cpu_temp_sysfs() -> Option<f64> {
    let paths = [
        "/sys/class/thermal/thermal_zone0/temp",
        "/sys/class/hwmon/hwmon0/temp1_input",
        "/sys/class/hwmon/hwmon1/temp1_input",
    ];
    for path in &paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(millideg) = content.trim().parse::<f64>() {
                return Some(millideg / 1000.0);
            }
        }
    }
    None
}

fn read_net_procfs() -> (u64, u64) {
    let content = match std::fs::read_to_string("/proc/net/dev") {
        Ok(c) => c,
        Err(_) => return (0, 0),
    };
    let mut total_rx = 0u64;
    let mut total_tx = 0u64;

    for line in content.lines().skip(2) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 10 {
            if let Ok(rx) = parts[1].parse::<u64>() {
                if let Ok(tx) = parts[9].parse::<u64>() {
                    if parts[0].trim_end_matches(':') != "lo" {
                        total_rx += rx;
                        total_tx += tx;
                    }
                }
            }
        }
    }

    (total_rx, total_tx)
}

fn read_net_interfaces_procfs() -> Vec<(String, NetTraffic)> {
    let content = std::fs::read_to_string("/proc/net/dev").ok();
    let mut interfaces = Vec::new();

    if let Some(content) = content {
        for line in content.lines().skip(2) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                let name = parts[0].trim_end_matches(':').to_string();
                if name != "lo" {
                    if let Ok(rx) = parts[1].parse::<u64>() {
                        if let Ok(tx) = parts[9].parse::<u64>() {
                            interfaces.push((
                                name,
                                NetTraffic { rx, tx },
                            ));
                        }
                    }
                }
            }
        }
    }

    interfaces
}

fn collect_disk_procfs() -> DiskMetrics {
    let content = match std::fs::read_to_string("/proc/mounts") {
        Ok(c) => c,
        Err(_) => return DiskMetrics { disks: Vec::new() },
    };

    let mut disk_info_vec = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let mount_point = parts[1].to_string();

        if !mount_point.starts_with('/') || seen.contains(&mount_point) {
            continue;
        }

        if mount_point.starts_with("/dev")
            || mount_point.starts_with("/sys")
            || mount_point.starts_with("/proc")
            || mount_point.starts_with("/run")
        {
            continue;
        }

        seen.insert(mount_point.clone());

        if let Some((total, used)) = statvfs_disk_usage(&mount_point) {
            let percent = if total > 0 {
                (used as f64 / total as f64) * 100.0
            } else {
                0.0
            };

            disk_info_vec.push(DiskInfo {
                mount_point,
                total,
                used,
                percent,
            });
        }
    }

    DiskMetrics {
        disks: disk_info_vec,
    }
}

fn statvfs_disk_usage(path: &str) -> Option<(u64, u64)> {
    use std::ffi::CString;
    use std::mem;

    let c_path = CString::new(path).ok()?;
    let mut stat: libc::statvfs = unsafe { mem::zeroed() };

    if unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) } == 0 {
        let frsize = stat.f_frsize as u64;
        let total = stat.f_blocks * frsize;
        let bfree = stat.f_bfree * frsize;
        let used = total.saturating_sub(bfree);
        Some((total, used))
    } else {
        None
    }
}

fn collect_battery_sysfs() -> BatteryMetrics {
    let capacity = std::fs::read_to_string("/sys/class/power_supply/BAT0/capacity")
        .ok()
        .and_then(|s| s.trim().parse::<f64>().ok());

    let charging = std::fs::read_to_string("/sys/class/power_supply/BAT0/status")
        .ok()
        .map(|s| s.trim() == "Charging");

    BatteryMetrics { capacity, charging }
}

struct SysfsGpuMonitor {
    driver: &'static str,
    hwmon_paths: Vec<String>,
}

impl SysfsGpuMonitor {
    fn new(driver: &'static str) -> Self {
        let mut hwmon_paths = Vec::new();
        if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name = name.to_string_lossy();
                if name.starts_with("card") {
                    let device_path = entry.path().join("device");
                    if let Ok(target) = device_path.join("driver").read_link() {
                        if target.to_string_lossy().contains(driver) {
                            if let Ok(hwmons) = std::fs::read_dir(device_path.join("hwmon")) {
                                for hwmon in hwmons.flatten() {
                                    hwmon_paths.push(hwmon.path().to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        SysfsGpuMonitor { driver, hwmon_paths }
    }

    fn collect(&self) -> Vec<GpuInfo> {
        if self.hwmon_paths.is_empty() {
            return Vec::new();
        }

        let vendor = match self.driver {
            "amdgpu" => GpuVendor::Amd,
            "i915" => GpuVendor::Intel,
            _ => GpuVendor::Unknown,
        };

        let mut gpus = Vec::new();
        for path in &self.hwmon_paths {
            let temp = std::fs::read_to_string(format!("{}/temp1_input", path))
                .ok()
                .and_then(|s| s.trim().parse::<f64>().ok())
                .map(|v| v / 1000.0);

            let name = std::fs::read_to_string(format!("{}/name", path))
                .unwrap_or_default()
                .trim()
                .to_string();

            gpus.push(GpuInfo {
                name: format!("{} ({})", self.driver, name),
                temperature: temp,
                usage: None,
                memory_total: None,
                memory_used: None,
                vendor: vendor.clone(),
            });
        }

        gpus
    }
}

struct NvidiaGpuMonitor {
    gpus: Vec<GpuInfo>,
}

impl NvidiaGpuMonitor {
    fn new() -> Option<Self> {
        let nvml = nvml_wrapper::Nvml::init().ok()?;
        let count = nvml.device_count().ok()?;
        let mut gpus = Vec::new();

        for i in 0..count {
            if let Ok(device) = nvml.device_by_index(i) {
                let name = device.name().unwrap_or_default();
                let temp = device
                    .temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)
                    .ok()
                    .map(|t| t as f64);
                let usage = device.utilization_rates().ok().map(|u| u.gpu as f64);
                if let Ok(mem_info) = device.memory_info() {
                    gpus.push(GpuInfo {
                        name,
                        temperature: temp,
                        usage,
                        memory_total: Some(mem_info.total),
                        memory_used: Some(mem_info.used),
                        vendor: GpuVendor::Nvidia,
                    });
                } else {
                    gpus.push(GpuInfo {
                        name,
                        temperature: temp,
                        usage,
                        memory_total: None,
                        memory_used: None,
                        vendor: GpuVendor::Nvidia,
                    });
                }
            }
        }

        std::mem::forget(nvml);
        Some(NvidiaGpuMonitor { gpus })
    }

    fn collect(&mut self) -> Vec<GpuInfo> {
        self.gpus.clone()
    }
}
