export interface CpuMetrics {
  total_usage: number;
  per_core_usage: number[];
  temperature: number | null;
  frequency: number | null;
  load_avg: [number, number, number];
}

export interface MemoryMetrics {
  total: number;
  used: number;
  cached: number;
  available: number;
  swap_total: number;
  swap_used: number;
  percent: number;
}

export interface DiskInfo {
  mount_point: string;
  total: number;
  used: number;
  percent: number;
}

export interface DiskMetrics {
  disks: DiskInfo[];
}

export interface NetTraffic {
  rx: number;
  tx: number;
}

export interface NetworkMetrics {
  interfaces: [string, NetTraffic][];
  total_rx: number;
  total_tx: number;
}

export type GpuVendor = "Nvidia" | "Amd" | "Intel" | "Unknown";

export interface GpuInfo {
  name: string;
  temperature: number | null;
  usage: number | null;
  memory_total: number | null;
  memory_used: number | null;
  vendor: GpuVendor;
}

export interface GpuMetrics {
  gpus: GpuInfo[];
}

export interface ProcessInfo {
  pid: number;
  name: string;
  cpu_usage: number;
  memory: number;
  memory_percent: number;
  state: string;
}

export interface BatteryMetrics {
  capacity: number | null;
  charging: boolean | null;
}

export interface SystemMetrics {
  cpu: CpuMetrics;
  memory: MemoryMetrics;
  disk: DiskMetrics;
  network: NetworkMetrics;
  gpu: GpuMetrics;
  processes: ProcessInfo[];
  battery: BatteryMetrics;
}

export interface MetricsHistorySnapshot {
  cpu_total: number[];
  cpu_per_core: number[][];
  ram_usage: number[];
  gpu_temperatures: number[][];
  net_rx: number[];
  net_tx: number[];
  cpu_temp: number[];
}

export interface MetricsUpdatePayload {
  metrics: SystemMetrics;
  history: MetricsHistorySnapshot;
}

export type AlertEvent =
  | "cputemperature" | "gputemperature" | "cpuusage"
  | "ramusage" | "diskusage" | "networkusage"
  | "batterylevel" | "processcount";

export interface AlertRule {
  event: AlertEvent;
  threshold: number;
  sound: string;
}

export interface AlertEventInfo {
  event: AlertEvent;
  current_value: number;
  threshold: number;
  sound: string;
}

export interface Config {
  settings: {
    update_interval_ms: number;
    theme: string;
  };
  alerts: {
    enabled: boolean;
    events: AlertRule[];
  };
}

export interface StatusPayload {
  cpu: number;
  ram: number;
  process_count: number;
  cpu_str: string;
  ram_str: string;
}

export function formatBytes(bytes: number): string {
  if (bytes >= 1_000_000_000) return `${(bytes / 1_000_000_000).toFixed(1)} GB`;
  if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(1)} MB`;
  if (bytes >= 1_000) return `${(bytes / 1_000).toFixed(1)} KB`;
  return `${bytes} B`;
}

export function formatBytesShort(bytes: number): string {
  if (bytes >= 1_000_000_000) return `${(bytes / 1_000_000_000).toFixed(1)} GB`;
  if (bytes >= 1_000_000) return `${(bytes / 1_000_000).toFixed(0)} MB`;
  if (bytes >= 1_000) return `${(bytes / 1_000).toFixed(0)} KB`;
  return `${bytes} B`;
}

const ALERT_LABELS: Record<AlertEvent, string> = {
  cputemperature: "CPU Temp",
  gputemperature: "GPU Temp",
  cpuusage: "CPU Usage",
  ramusage: "RAM Usage",
  diskusage: "Disk Usage",
  networkusage: "Network",
  batterylevel: "Battery",
  processcount: "Process Count",
};

export function alertLabel(event: AlertEvent): string {
  return ALERT_LABELS[event];
}

export const SOUND_PRESETS = ["beep", "alert", "alarm", "notification", "critical", "warnsiren", "chime"];

export const ALL_EVENTS: AlertEvent[] = [
  "cputemperature", "gputemperature", "cpuusage",
  "ramusage", "diskusage", "networkusage",
  "batterylevel", "processcount",
];
