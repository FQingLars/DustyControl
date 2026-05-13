<script lang="ts">
  import { onMount } from "svelte";

  let { data = [], color = "#22c55e", height = 60, maxVal = 100 }: {
    data?: number[];
    color?: string;
    height?: number;
    maxVal?: number;
  } = $props();

  let canvas: HTMLCanvasElement;

  function draw() {
    const c = canvas;
    if (!c) return;
    const ctx = c.getContext("2d")!;
    const dpr = window.devicePixelRatio || 1;
    const w = c.width / dpr;
    const h = c.height / dpr;

    ctx.clearRect(0, 0, w, h);
    if (data.length < 2) return;

    const max = Math.max(...data, maxVal, 1);
    const pad = 4;
    const dw = w - pad * 2;
    const dh = h - pad * 2;

    ctx.strokeStyle = color;
    ctx.lineWidth = 2;
    ctx.lineJoin = "round";
    ctx.beginPath();

    for (let i = 0; i < data.length; i++) {
      const x = pad + (i / (data.length - 1)) * dw;
      const y = pad + dh - (data[i] / max) * dh;
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();

    const ly = pad + dh;
    ctx.fillStyle = color + "30";
    ctx.lineTo(pad + dw, ly);
    ctx.lineTo(pad, ly);
    ctx.closePath();
    ctx.fill();
  }

  $effect(() => { data; draw(); });

  onMount(() => {
    const dpr = window.devicePixelRatio || 1;
    const rect = canvas.getBoundingClientRect();
    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    const ctx = canvas.getContext("2d")!;
    ctx.scale(dpr, dpr);
    draw();
  });
</script>

<canvas bind:this={canvas} style="width:100%;height:{height}px;border-radius:4px;background:#0f172a"></canvas>
