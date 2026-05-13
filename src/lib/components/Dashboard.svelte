<script lang="ts">
  import CircularGauge from "./CircularGauge.svelte";
  import MiniChart from "./MiniChart.svelte";
  import type { SystemMetrics, MetricsHistorySnapshot } from "../types";
  import { formatBytesShort } from "../types";

  let { metrics, history }: {
    metrics: SystemMetrics | null;
    history: MetricsHistorySnapshot | null;
  } = $props();
</script>

<div class="dashboard">
  <div class="gauges-row">
    {#if metrics}
      <div class="gauge-card">
        <CircularGauge value={metrics.cpu.total_usage} label="CPU" size={100} strokeWidth={6} />
        <div class="gauge-info">
          {#if metrics.cpu.temperature !== null}<span class="info-temp">{metrics.cpu.temperature.toFixed(0)}°C</span>{/if}
          {#if metrics.cpu.frequency !== null}<span class="info-sub">{metrics.cpu.frequency} MHz</span>{/if}
        </div>
      </div>

      <div class="gauge-card">
        <CircularGauge value={metrics.memory.percent} label="RAM" size={100} strokeWidth={6} />
        <div class="gauge-info">
          <span class="info-temp">{(metrics.memory.used / 1_000_000_000).toFixed(1)} GB</span>
          <span class="info-sub">/ {(metrics.memory.total / 1_000_000_000).toFixed(1)} GB</span>
        </div>
      </div>

      <div class="gauge-card">
        {#if metrics.gpu.gpus[0]}
          {@const g = metrics.gpu.gpus[0]}
          <CircularGauge value={g.usage ?? 0} label={g.name.slice(0, 10)} size={100} strokeWidth={6} />
          <div class="gauge-info">
            {#if g.temperature !== null}<span class="info-temp">{g.temperature.toFixed(0)}°C</span>{/if}
            {#if g.memory_used !== null}<span class="info-sub">{formatBytesShort(g.memory_used)}</span>{/if}
          </div>
        {:else}
          <CircularGauge value={0} label="GPU" size={100} strokeWidth={6} />
          <div class="gauge-info"><span class="info-sub">No GPU</span></div>
        {/if}
      </div>

      <div class="gauge-card">
        {#if metrics.disk.disks[0]}
          {@const d = metrics.disk.disks[0]}
          <CircularGauge value={d.percent} label={d.mount_point.replace("/", "") || "Disk"} size={100} strokeWidth={6} />
          <div class="gauge-info">
            <span class="info-temp">{(d.used / 1_000_000_000).toFixed(1)} GB</span>
            <span class="info-sub">/ {(d.total / 1_000_000_000).toFixed(0)} GB</span>
          </div>
        {:else}
          <CircularGauge value={0} label="Disk" size={100} strokeWidth={6} />
          <div class="gauge-info"><span class="info-sub">No disks</span></div>
        {/if}
      </div>

      <div class="gauge-card">
        <CircularGauge value={metrics.battery.capacity ?? 0} label="BAT" size={100} strokeWidth={6} />
        <div class="gauge-info">
          {#if metrics.battery.charging !== null}
            <span class="info-temp">{metrics.battery.charging ? "Charging" : "Discharging"}</span>
          {:else}
            <span class="info-sub">No battery</span>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  <div class="charts-row">
    {#if history}
      <div class="chart-card">
        <div class="chart-title">CPU</div>
        <MiniChart data={[...history.cpu_total].slice(-60)} color="#22d3ee" height={55} maxVal={100} />
      </div>
      <div class="chart-card">
        <div class="chart-title">RAM</div>
        <MiniChart data={[...history.ram_usage].slice(-60)} color="#22c55e" height={55} maxVal={100} />
      </div>
      <div class="chart-card">
        <div class="chart-title">CPU Temp</div>
        <MiniChart data={[...history.cpu_temp].slice(-60)} color="#f97316" height={55} />
      </div>
    {/if}
    {#if metrics}
      <div class="chart-card wide">
        <div class="chart-title">Network</div>
        <div class="net-rates">
          <span class="net-down">↓ {formatBytesShort(metrics.network.total_rx)}/s</span>
          <span class="net-up">↑ {formatBytesShort(metrics.network.total_tx)}/s</span>
        </div>
        {#if history}
          <MiniChart data={[...history.net_rx].slice(-60)} color="#3b82f6" height={40} />
        {/if}
      </div>
    {/if}
  </div>

  {#if metrics}
    <div class="sys-info">
      <div class="stat-item"><span>Processes</span><b>{metrics.processes.length}</b></div>
      <div class="stat-item"><span>Load Avg</span><b>{metrics.cpu.load_avg[0].toFixed(2)}</b></div>
      <div class="stat-item"><span>Cores</span><b>{metrics.cpu.per_core_usage.length}</b></div>
    </div>
  {/if}
</div>

<style>
  .dashboard { padding: 8px; }
  .gauges-row { display: flex; gap: 12px; justify-content: center; flex-wrap: wrap; }
  .gauge-card {
    background: #1e293b;
    border-radius: 12px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    min-width: 120px;
  }
  .gauge-info { margin-top: 6px; text-align: center; }
  .info-temp { font-size: 0.8em; font-weight: 600; color: #e2e8f0; }
  .info-sub { font-size: 0.65em; color: #64748b; display: block; }

  .charts-row { display: flex; gap: 8px; margin-top: 10px; flex-wrap: wrap; }
  .chart-card {
    background: #1e293b;
    border-radius: 10px;
    padding: 8px 10px;
    flex: 1;
    min-width: 140px;
  }
  .chart-card.wide { flex: 2; min-width: 200px; }
  .chart-title { font-size: 0.7em; color: #94a3b8; margin-bottom: 4px; }

  .net-rates { display: flex; gap: 12px; margin-bottom: 4px; }
  .net-down { font-size: 0.75em; color: #60a5fa; }
  .net-up { font-size: 0.75em; color: #f87171; }

  .sys-info {
    display: flex;
    gap: 20px;
    justify-content: center;
    margin-top: 10px;
    background: #1e293b;
    border-radius: 10px;
    padding: 8px 16px;
    font-size: 0.75em;
  }
  .stat-item { display: flex; gap: 6px; color: #94a3b8; }
  .stat-item b { color: #e2e8f0; }
</style>
