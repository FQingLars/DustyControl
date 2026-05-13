export class AudioManager {
  private ctx: AudioContext | null = null;

  private ctx_get(): AudioContext {
    if (!this.ctx) this.ctx = new AudioContext();
    return this.ctx;
  }

  play(preset: string) {
    const ctx = this.ctx_get();
    if (ctx.state === "suspended") ctx.resume();

    switch (preset) {
      case "beep": this.tone(ctx, 880, 0.3, 0.2); break;
      case "alert": this.tone(ctx, 660, 0.4, 0.4); break;
      case "alarm": this.tone(ctx, 440, 0.5, 0.6); break;
      case "notification": this.tone(ctx, 1047, 0.2, 0.12); break;
      case "critical": this.tone(ctx, 880, 0.6, 0.8); break;
      case "warnsiren": this.sweep(ctx, 660, 880, 0.4, 0.5); break;
      case "chime": this.tone(ctx, 659, 0.3, 0.3); break;
    }
  }

  private tone(ctx: AudioContext, freq: number, vol: number, dur: number) {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = "sine";
    osc.frequency.value = freq;
    gain.gain.value = vol;
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + dur);
    osc.connect(gain).connect(ctx.destination);
    osc.start(ctx.currentTime);
    osc.stop(ctx.currentTime + dur);
  }

  private sweep(ctx: AudioContext, f0: number, f1: number, vol: number, dur: number) {
    const osc = ctx.createOscillator();
    const gain = ctx.createGain();
    osc.type = "sine";
    osc.frequency.setValueAtTime(f0, ctx.currentTime);
    osc.frequency.linearRampToValueAtTime(f1, ctx.currentTime + dur);
    gain.gain.value = vol;
    gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + dur);
    osc.connect(gain).connect(ctx.destination);
    osc.start(ctx.currentTime);
    osc.stop(ctx.currentTime + dur);
  }
}

let _inst: AudioManager | null = null;
export function getAudio(): AudioManager {
  if (!_inst) _inst = new AudioManager();
  return _inst;
}
