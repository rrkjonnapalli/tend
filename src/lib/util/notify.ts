// Desktop notification and a short chime, used when a timer ends.
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

const BEEP_STARTS = [0, 0.25];
const BEEP_HZ = 880;
const BEEP_PEAK_AT = 0.02;
const BEEP_FADE_AT = 0.2;
const BEEP_STOP_AT = 0.22;
const SILENT_GAIN = 0.0001;
const PEAK_GAIN = 0.2;
const CLOSE_AFTER_MS = 800;

/** Asks macOS / Windows / Linux for permission once; later calls just report the answer. */
export async function allowNotifications(): Promise<boolean> {
  if (await isPermissionGranted()) return true;
  return (await requestPermission()) === 'granted';
}

export async function notify({ title, body }: { title: string; body: string }) {
  if (await allowNotifications()) sendNotification({ title, body });
}

/** Two soft beeps made with Web Audio, so no sound file is needed. */
export function chime() {
  const ctx = new AudioContext();
  for (const at of BEEP_STARTS) beep(ctx, ctx.currentTime + at);
  setTimeout(() => ctx.close(), CLOSE_AFTER_MS);
}

function beep(ctx: AudioContext, start: number) {
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.frequency.value = BEEP_HZ;
  gain.gain.setValueAtTime(SILENT_GAIN, start);
  gain.gain.exponentialRampToValueAtTime(PEAK_GAIN, start + BEEP_PEAK_AT);
  gain.gain.exponentialRampToValueAtTime(SILENT_GAIN, start + BEEP_FADE_AT);
  osc.connect(gain).connect(ctx.destination);
  osc.start(start);
  osc.stop(start + BEEP_STOP_AT);
}
