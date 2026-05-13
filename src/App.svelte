<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import Dashboard from "./lib/components/Dashboard.svelte";
  import Processes from "./lib/components/Processes.svelte";
  import Alerts from "./lib/components/Alerts.svelte";
  import Settings from "./lib/components/Settings.svelte";
  import { getAudio } from "./lib/audio";
  import type { SystemMetrics, MetricsHistorySnapshot, AlertEventInfo, Config, StatusPayload } from "./lib/types";

  type Tab = "dashboard" | "processes" | "alerts" | "settings";

  let activeTab = $state<Tab>("dashboard");
  let metrics = $state<SystemMetrics | null>(null);
  let history = $state<MetricsHistorySnapshot | null>(null);
  let alerts = $state<AlertEventInfo[]>([]);
  let config = $state<Config | null>(null);
  let status = $state<StatusPayload | null>(null);
  let processes = $state<import("./lib/types").ProcessInfo[]>([]);

  let unlistenMetrics: (() => void) | undefined;
  let unlistenAlert: (() => void) | undefined;
  let unlistenStatus: (() => void) | undefined;

  onMount(async () => {
    try {
      config = await invoke<Config>("get_config");
    } catch (e) { console.error(e); }

    unlistenMetrics = await listen<{ metrics: SystemMetrics; history: MetricsHistorySnapshot }>("metrics-update", (e) => {
      metrics = e.payload.metrics;
      history = e.payload.history;
      processes = e.payload.metrics.processes;
    });

    unlistenAlert = await listen<AlertEventInfo>("alert-triggered", (e) => {
      alerts = [...alerts, e.payload];
      if (alerts.length > 50) alerts = alerts.slice(-50);
      if (config?.alerts.enabled !== false) {
        getAudio().play(e.payload.sound);
      }
    });

    unlistenStatus = await listen<StatusPayload>("status-update", (e) => {
      status = e.payload;
    });
  });

  onDestroy(() => {
    unlistenMetrics?.();
    unlistenAlert?.();
    unlistenStatus?.();
  });

  const tabs: { id: Tab; label: string }[] = [
    { id: "dashboard", label: "Dashboard" },
    { id: "processes", label: "Processes" },
    { id: "alerts", label: "Alerts" },
    { id: "settings", label: "Settings" },
  ];
</script>

<div class="app">
  <header>
    <span class="logo">DustyControl</span>
    <nav>
      {#each tabs as tab}
        <button
          class="tab-btn"
          class:active={activeTab === tab.id}
          onclick={() => activeTab = tab.id}
        >{tab.label}</button>
      {/each}
    </nav>
  </header>

  <main>
    {#if activeTab === "dashboard"}
      <Dashboard {metrics} {history} />
    {:else if activeTab === "processes"}
      <Processes {processes} />
    {:else if activeTab === "alerts"}
      <Alerts {alerts} />
    {:else if activeTab === "settings"}
      <Settings {config} />
    {/if}
  </main>

  <footer>
    {#if status}
      <span>CPU: {status.cpu_str}</span>
      <span>RAM: {status.ram_str}</span>
      <span>Processes: {status.process_count}</span>
    {/if}
  </footer>
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: #0f172a;
    color: #e2e8f0;
    font-family: 'Inter', 'Segoe UI', system-ui, -apple-system, sans-serif;
  }

  header {
    display: flex;
    align-items: center;
    padding: 8px 16px;
    background: #1e293b;
    border-bottom: 1px solid #334155;
    gap: 20px;
    flex-shrink: 0;
  }

  .logo {
    font-weight: 700;
    font-size: 1em;
    color: #22d3ee;
  }

  nav { display: flex; gap: 4px; }

  .tab-btn {
    background: none;
    border: none;
    color: #64748b;
    padding: 6px 14px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.8em;
    transition: all 0.15s;
  }
  .tab-btn:hover { color: #e2e8f0; background: #334155; }
  .tab-btn.active { color: #22d3ee; background: #0f172a; font-weight: 600; }

  main {
    flex: 1;
    overflow-y: auto;
  }

  footer {
    display: flex;
    gap: 20px;
    padding: 6px 16px;
    background: #1e293b;
    border-top: 1px solid #334155;
    font-size: 0.7em;
    color: #64748b;
    flex-shrink: 0;
  }
</style>
