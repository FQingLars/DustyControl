<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { Config, AlertRule, AlertEvent } from "../types";
  import { ALL_EVENTS, SOUND_PRESETS, alertLabel } from "../types";

  let { config }: { config: Config | null } = $props();

  let interval = $state("1000");
  let enabled = $state(true);
  let rules = $state<AlertRule[]>([]);
  let saving = $state(false);
  let saved = $state(false);

  let newEvent = $state<AlertEvent>("cpuusage");
  let newThreshold = $state("90");
  let newSound = $state("beep");

  $effect(() => {
    if (config) {
      interval = String(config.settings.update_interval_ms);
      enabled = config.alerts.enabled;
      rules = config.alerts.events;
    }
  });

  async function save() {
    saving = true;
    const cfg: Config = {
      settings: { update_interval_ms: parseInt(interval) || 1000, theme: "dark" },
      alerts: { enabled, events: rules },
    };
    try {
      await invoke("save_config", { cfg });
      saved = true;
      setTimeout(() => saved = false, 2000);
    } catch (e) {
      console.error(e);
    }
    saving = false;
  }

  function addRule() {
    rules = [...rules, { event: newEvent, threshold: parseFloat(newThreshold) || 0, sound: newSound }];
  }

  function removeRule(i: number) {
    rules = rules.filter((_, idx) => idx !== i);
  }
</script>

<div class="settings">
  <div class="section">
    <h3>General Settings</h3>
    <div class="field">
      <label for="interval">Update interval (ms)</label>
      <input id="interval" type="number" bind:value={interval} min="200" max="10000" />
    </div>
    <div class="field">
      <label for="alerts-enabled">Alerts enabled</label>
      <button class="toggle" class:on={enabled} onclick={() => enabled = !enabled} aria-label="Toggle alerts">
        <div class="toggle-knob"></div>
      </button>
    </div>
  </div>

  <div class="section">
    <h3>Alert Rules</h3>
    <div class="add-rule">
      <select bind:value={newEvent}>
        {#each ALL_EVENTS as ev}
          <option value={ev}>{alertLabel(ev)}</option>
        {/each}
      </select>
      <input type="number" bind:value={newThreshold} step="0.1" placeholder="Threshold" />
      <select bind:value={newSound}>
        {#each SOUND_PRESETS as s}
          <option value={s}>{s}</option>
        {/each}
      </select>
      <button class="btn-add" onclick={addRule}>+</button>
    </div>
    {#each rules as rule, i}
      <div class="rule-row">
        <span class="rule-event">{alertLabel(rule.event)}</span>
        <span class="rule-threshold">&gt; {rule.threshold}</span>
        <span class="rule-sound">{rule.sound}</span>
        <button class="btn-del" onclick={() => removeRule(i)}>×</button>
      </div>
    {/each}
  </div>

  <button class="btn-save" onclick={save} disabled={saving}>
    {saving ? "Saving..." : saved ? "✓ Saved" : "Save Configuration"}
  </button>
</div>

<style>
  .settings { padding: 12px; max-width: 500px; }
  .section { margin-bottom: 20px; }
  .section h3 { font-size: 0.85em; color: #94a3b8; margin: 0 0 10px 0; }
  .field { display: flex; justify-content: space-between; align-items: center; margin-bottom: 10px; }
  .field label { font-size: 0.8em; color: #e2e8f0; }
  .field input { background: #1e293b; border: 1px solid #334155; border-radius: 6px; padding: 6px 10px; color: #e2e8f0; font-size: 0.8em; width: 120px; }

  .toggle {
    width: 40px; height: 22px; background: #475569; border-radius: 11px;
    cursor: pointer; position: relative; transition: background 0.2s;
  }
  .toggle.on { background: #22c55e; }
  .toggle-knob {
    width: 18px; height: 18px; background: #e2e8f0; border-radius: 50%;
    position: absolute; top: 2px; left: 2px;
    transition: transform 0.2s;
  }
  .toggle.on .toggle-knob { transform: translateX(18px); }

  .add-rule { display: flex; gap: 6px; margin-bottom: 10px; }
  .add-rule select, .add-rule input {
    background: #1e293b; border: 1px solid #334155; border-radius: 6px;
    padding: 5px 8px; color: #e2e8f0; font-size: 0.75em;
  }
  .add-rule select { max-width: 130px; }
  .add-rule input { width: 70px; }

  .btn-add {
    background: #1e3a5f; color: #93c5fd; border: none; border-radius: 6px;
    padding: 5px 12px; cursor: pointer; font-size: 0.8em;
  }
  .btn-add:hover { background: #1e40af; }

  .rule-row {
    display: flex; align-items: center; gap: 10px; padding: 6px 10px;
    background: #1e293b; border-radius: 6px; margin-bottom: 4px; font-size: 0.75em;
  }
  .rule-event { color: #22d3ee; width: 100px; }
  .rule-threshold { color: #f59e0b; width: 60px; }
  .rule-sound { color: #94a3b8; width: 80px; }
  .btn-del {
    background: none; border: none; color: #ef4444; cursor: pointer;
    font-size: 1.1em; padding: 0 4px;
  }

  .btn-save {
    background: #1d4ed8; color: #bfdbfe; border: none; border-radius: 8px;
    padding: 8px 24px; font-size: 0.8em; cursor: pointer; margin-top: 10px;
  }
  .btn-save:hover { background: #2563eb; }
  .btn-save:disabled { opacity: 0.5; }
</style>
