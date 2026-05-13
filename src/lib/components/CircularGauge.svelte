<script lang="ts">
  let { value = 0, label = "", size = 120, strokeWidth = 8, suffix = "%" }: {
    value?: number;
    label?: string;
    size?: number;
    strokeWidth?: number;
    suffix?: string;
  } = $props();

  const r = $derived((size - strokeWidth) / 2);
  const circumference = $derived(2 * Math.PI * r);
  const offset = $derived(circumference - (Math.min(value, 100) / 100) * circumference);
  const center = $derived(size / 2);

  const color = $derived(
    value >= 90 ? "#ef4444" :
    value >= 70 ? "#f59e0b" :
    value >= 50 ? "#eab308" :
    "#22c55e"
  );

  const numCores = $derived(label.includes("CPU") ? 0 : 0);
</script>

<div class="gauge-wrap" style="width:{size}px;height:{size}px">
  <svg width={size} height={size} viewBox="0 0 {size} {size}">
    <circle
      cx={center} cy={center} r={r}
      fill="none" stroke="#1e293b" stroke-width={strokeWidth}
    />
    <circle
      cx={center} cy={center} r={r}
      fill="none" stroke={color} stroke-width={strokeWidth}
      stroke-linecap="round"
      stroke-dasharray={circumference}
      stroke-dashoffset={offset}
      transform="rotate(-90 {center} {center})"
      style="transition: stroke-dashoffset 0.4s ease"
    />
  </svg>
  <div class="gauge-text">
    <span class="gauge-val">{Math.round(value)}{suffix}</span>
    {#if label}
      <span class="gauge-label">{label}</span>
    {/if}
  </div>
</div>

<style>
  .gauge-wrap {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .gauge-wrap svg { position: absolute; top: 0; left: 0; }
  .gauge-text {
    display: flex;
    flex-direction: column;
    align-items: center;
    z-index: 1;
  }
  .gauge-val {
    font-size: 1.2em;
    font-weight: 700;
    color: #e2e8f0;
    line-height: 1;
  }
  .gauge-label {
    font-size: 0.65em;
    color: #94a3b8;
    margin-top: 2px;
  }
</style>
