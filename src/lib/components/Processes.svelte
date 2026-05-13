<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { ProcessInfo } from "../types";
  import { formatBytes } from "../types";

  let { processes = [] }: { processes?: ProcessInfo[] } = $props();

  let loading = $state(false);

  async function kill(pid: number) {
    loading = true;
    try { await invoke("kill_process", { pid }); } catch (e) { console.error(e); }
    loading = false;
  }

  async function renice(pid: number, d: number) {
    loading = true;
    try { await invoke("renice_process", { pid, delta: d }); } catch (e) { console.error(e); }
    loading = false;
  }
</script>

<div class="procs">
  <div class="proc-table">
    <div class="proc-header">
      <span class="col-pid">PID</span>
      <span class="col-name">Name</span>
      <span class="col-cpu">CPU%</span>
      <span class="col-mem">Memory</span>
      <span class="col-memp">Mem%</span>
      <span class="col-state">State</span>
      <span class="col-acts">Actions</span>
    </div>
    {#each processes as proc (proc.pid)}
      <div class="proc-row">
        <span class="col-pid">{proc.pid}</span>
        <span class="col-name" title={proc.name}>{proc.name}</span>
        <span class="col-cpu">{proc.cpu_usage.toFixed(1)}</span>
        <span class="col-mem">{formatBytes(proc.memory)}</span>
        <span class="col-memp">{proc.memory_percent.toFixed(1)}</span>
        <span class="col-state">{proc.state}</span>
        <span class="col-acts">
          <button class="btn-kill" onclick={() => kill(proc.pid)} disabled={loading}>KILL</button>
          <button class="btn-nice" onclick={() => renice(proc.pid, 5)} disabled={loading}>+5</button>
          <button class="btn-nice" onclick={() => renice(proc.pid, -5)} disabled={loading}>-5</button>
        </span>
      </div>
    {/each}
  </div>
</div>

<style>
  .procs { padding: 8px; overflow-y: auto; max-height: calc(100vh - 180px); }
  .proc-table { width: 100%; font-size: 0.75em; }
  .proc-header, .proc-row {
    display: flex;
    padding: 6px 8px;
    gap: 4px;
    align-items: center;
  }
  .proc-header { color: #64748b; border-bottom: 1px solid #334155; position: sticky; top: 0; background: #0f172a; }
  .proc-row:hover { background: #1e293b; border-radius: 4px; }
  .col-pid { width: 60px; color: #94a3b8; }
  .col-name { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: #e2e8f0; }
  .col-cpu { width: 60px; text-align: right; color: #22d3ee; }
  .col-mem { width: 80px; text-align: right; color: #e2e8f0; }
  .col-memp { width: 55px; text-align: right; color: #94a3b8; }
  .col-state { width: 80px; color: #94a3b8; }
  .col-acts { width: 140px; display: flex; gap: 4px; }
  .btn-kill {
    background: #7f1d1d; color: #fca5a5; border: none; border-radius: 4px;
    padding: 2px 8px; font-size: 0.7em; cursor: pointer;
  }
  .btn-kill:hover { background: #991b1b; }
  .btn-nice {
    background: #1e3a5f; color: #93c5fd; border: none; border-radius: 4px;
    padding: 2px 8px; font-size: 0.7em; cursor: pointer;
  }
  .btn-nice:hover { background: #1e40af; }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
