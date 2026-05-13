<script lang="ts">
  import type { AlertEventInfo } from "../types";
  import { alertLabel } from "../types";

  let { alerts = [] }: { alerts?: AlertEventInfo[] } = $props();
</script>

<div class="alerts">
  {#if alerts.length === 0}
    <div class="empty">No alerts triggered</div>
  {:else}
    {#each [...alerts].reverse() as alert, i}
      <div class="alert-item" class:new={i < 3}>
        <div class="alert-left">
          <span class="alert-event">{alertLabel(alert.event)}</span>
          <span class="alert-values">
            {alert.current_value.toFixed(1)} / {alert.threshold.toFixed(1)}
          </span>
        </div>
        <span class="alert-sound">{alert.sound}</span>
      </div>
    {/each}
  {/if}
</div>

<style>
  .alerts { padding: 8px; overflow-y: auto; max-height: calc(100vh - 180px); }
  .empty { color: #64748b; text-align: center; padding: 40px; font-size: 0.85em; }
  .alert-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #1e293b;
    border-left: 3px solid #ef4444;
    border-radius: 6px;
    padding: 8px 12px;
    margin-bottom: 6px;
  }
  .alert-item.new {
    border-left-color: #f59e0b;
  }
  .alert-left { display: flex; flex-direction: column; gap: 2px; }
  .alert-event { font-weight: 600; color: #e2e8f0; font-size: 0.85em; }
  .alert-values { font-size: 0.7em; color: #94a3b8; }
  .alert-sound {
    font-size: 0.7em;
    color: #f59e0b;
    background: #451a03;
    padding: 2px 8px;
    border-radius: 4px;
  }
</style>
