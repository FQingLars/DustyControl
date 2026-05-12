use std::collections::VecDeque;

pub const METRICS_HISTORY_LEN: usize = 120;
pub const UPDATE_INTERVAL_MS: u64 = 1000;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CpuMetrics {
    pub total_usage: f64,
    pub per_core_usage: Vec<f64>,
    pub temperature: Option<f64>,
    pub frequency: Option<u64>,
    pub load_avg: [f64; 3],
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MemoryMetrics {
    pub total: u64,
    pub used: u64,
    pub cached: u64,
    pub available: u64,
    pub swap_total: u64,
    pub swap_used: u64,
    pub percent: f64,
}

#[derive(Clone, Debug)]
pub struct DiskMetrics {
    pub disks: Vec<DiskInfo>,
}

#[derive(Clone, Debug)]
pub struct DiskInfo {
    pub mount_point: String,
    pub total: u64,
    pub used: u64,
    pub percent: f64,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct NetTraffic {
    pub rx: u64,
    pub tx: u64,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct NetworkMetrics {
    pub interfaces: Vec<(String, NetTraffic)>,
    pub total_rx: u64,
    pub total_tx: u64,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct GpuInfo {
    pub name: String,
    pub temperature: Option<f64>,
    pub usage: Option<f64>,
    pub memory_total: Option<u64>,
    pub memory_used: Option<u64>,
    pub vendor: GpuVendor,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct GpuMetrics {
    pub gpus: Vec<GpuInfo>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub memory_percent: f32,
    pub state: String,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct BatteryMetrics {
    pub capacity: Option<f64>,
    pub charging: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct SystemMetrics {
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
    pub gpu: GpuMetrics,
    pub processes: Vec<ProcessInfo>,
    pub battery: BatteryMetrics,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct MetricsHistory {
    pub timestamps: VecDeque<chrono::DateTime<chrono::Utc>>,
    pub cpu_total: VecDeque<f64>,
    pub cpu_per_core: Vec<VecDeque<f64>>,
    pub ram_usage: VecDeque<f64>,
    pub gpu_temperatures: Vec<VecDeque<f64>>,
    pub net_rx: VecDeque<f64>,
    pub net_tx: VecDeque<f64>,
    pub cpu_temp: VecDeque<f64>,
}

impl MetricsHistory {
    pub fn new(core_count: usize, gpu_count: usize) -> Self {
        Self {
            timestamps: VecDeque::with_capacity(METRICS_HISTORY_LEN),
            cpu_total: VecDeque::with_capacity(METRICS_HISTORY_LEN),
            cpu_per_core: vec![VecDeque::with_capacity(METRICS_HISTORY_LEN); core_count],
            ram_usage: VecDeque::with_capacity(METRICS_HISTORY_LEN),
            gpu_temperatures: vec![VecDeque::with_capacity(METRICS_HISTORY_LEN); gpu_count],
            net_rx: VecDeque::with_capacity(METRICS_HISTORY_LEN),
            net_tx: VecDeque::with_capacity(METRICS_HISTORY_LEN),
            cpu_temp: VecDeque::with_capacity(METRICS_HISTORY_LEN),
        }
    }

    pub fn push(&mut self, metrics: &SystemMetrics) {
        let now = chrono::Utc::now();
        self.timestamps.push_back(now);
        if self.timestamps.len() > METRICS_HISTORY_LEN {
            self.timestamps.pop_front();
        }

        self.cpu_total.push_back(metrics.cpu.total_usage);
        if self.cpu_total.len() > METRICS_HISTORY_LEN {
            self.cpu_total.pop_front();
        }

        for (i, usage) in metrics.cpu.per_core_usage.iter().enumerate() {
            if i < self.cpu_per_core.len() {
                self.cpu_per_core[i].push_back(*usage);
                if self.cpu_per_core[i].len() > METRICS_HISTORY_LEN {
                    self.cpu_per_core[i].pop_front();
                }
            }
        }

        self.ram_usage.push_back(metrics.memory.percent);
        if self.ram_usage.len() > METRICS_HISTORY_LEN {
            self.ram_usage.pop_front();
        }

        self.cpu_temp.push_back(metrics.cpu.temperature.unwrap_or(0.0));
        if self.cpu_temp.len() > METRICS_HISTORY_LEN {
            self.cpu_temp.pop_front();
        }

        for (i, gpu) in metrics.gpu.gpus.iter().enumerate() {
            if i < self.gpu_temperatures.len() {
                self.gpu_temperatures[i].push_back(gpu.temperature.unwrap_or(0.0));
                if self.gpu_temperatures[i].len() > METRICS_HISTORY_LEN {
                    self.gpu_temperatures[i].pop_front();
                }
            }
        }

        let net_rx = metrics.network.total_rx as f64;
        let net_tx = metrics.network.total_tx as f64;
        self.net_rx.push_back(net_rx);
        if self.net_rx.len() > METRICS_HISTORY_LEN {
            self.net_rx.pop_front();
        }
        self.net_tx.push_back(net_tx);
        if self.net_tx.len() > METRICS_HISTORY_LEN {
            self.net_tx.pop_front();
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct AlertRule {
    pub event: AlertEvent,
    pub threshold: f64,
    pub sound: String,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum AlertEvent {
    #[serde(rename = "cputemperature")]
    CpuTemperature,
    #[serde(rename = "gputemperature")]
    GpuTemperature,
    #[serde(rename = "cpuusage")]
    CpuUsage,
    #[serde(rename = "ramusage")]
    RamUsage,
    #[serde(rename = "diskusage")]
    DiskUsage,
    #[serde(rename = "networkusage")]
    NetworkUsage,
    #[serde(rename = "batterylevel")]
    BatteryLevel,
    #[serde(rename = "processcount")]
    ProcessCount,
}

impl std::fmt::Display for AlertEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertEvent::CpuTemperature => write!(f, "cputemperature"),
            AlertEvent::GpuTemperature => write!(f, "gputemperature"),
            AlertEvent::CpuUsage => write!(f, "cpuusage"),
            AlertEvent::RamUsage => write!(f, "ramusage"),
            AlertEvent::DiskUsage => write!(f, "diskusage"),
            AlertEvent::NetworkUsage => write!(f, "networkusage"),
            AlertEvent::BatteryLevel => write!(f, "batterylevel"),
            AlertEvent::ProcessCount => write!(f, "processcount"),
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct AlertEventInfo {
    pub event: AlertEvent,
    pub current_value: f64,
    pub threshold: f64,
    pub sound: String,
}
