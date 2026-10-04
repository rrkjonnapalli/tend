// Timer durations: 'mm:ss' (minutes any length, seconds 0–59) or a bare number of minutes.

export type Duration = { ok: true; ms: number; label: string } | { ok: false; error: string };

export function parseDuration(text: string): Duration {
  const v = text.trim();
  let m = v.match(/^(\d+)$/);
  if (m) {
    const min = Number(m[1]);
    return min > 0 ? ok(min, 0) : { ok: false, error: 'must be longer than 0' };
  }
  m = v.match(/^(\d*):(\d{1,2})$/);
  if (!m) return { ok: false, error: 'use mm:ss, e.g. 05:00' };
  const [min, sec] = [Number(m[1] || 0), Number(m[2])];
  if (sec > 59) return { ok: false, error: 'seconds must be 0–59' };
  if (min === 0 && sec === 0) return { ok: false, error: 'must be longer than 0' };
  return ok(min, sec);
}

function ok(min: number, sec: number): Duration {
  return { ok: true, ms: (min * 60 + sec) * 1000, label: `${min}:${String(sec).padStart(2, '0')}` };
}

/** Milliseconds left → 'm:ss' (minutes grow past 59). */
export function countdown(ms: number): string {
  const sec = Math.max(0, Math.ceil(ms / 1000));
  return `${Math.floor(sec / 60)}:${String(sec % 60).padStart(2, '0')}`;
}
