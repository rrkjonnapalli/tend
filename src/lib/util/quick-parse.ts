// Phrases the ⌘K palette turns into things: "task: …", "note: …", "person: …", "timer 10m …".
import type { Phrase } from '#lib/types/quick-parse.type.ts';


const cap = (t: string) => t.charAt(0).toUpperCase() + t.slice(1);
const UNIT_MS: Record<string, number> = { h: 3_600_000, m: 60_000, s: 1000 };

/** "1h 30m", "10m", "90s", "5m30s" → milliseconds. */
export function parseSpan(text: string): number | null {
  const parts = [...text.matchAll(/(\d+)\s*([hms])/gi)];
  if (!parts.length || parts.map((p) => p[0]).join('').replace(/\s/g, '') !== text.replace(/\s/g, '')) return null;
  const ms = parts.reduce((sum, p) => sum + Number(p[1]) * UNIT_MS[p[2].toLowerCase()], 0);
  return ms > 0 ? ms : null;
}

export function parsePhrase(input: string): Phrase | null {
  const q = input.trim();
  let m = q.match(/^task:?\s+(.+)$/i);
  if (m) return { kind: 'task', title: cap(m[1].trim()) };
  m = q.match(/^note:?\s+(.+)$/i);
  if (m) return { kind: 'note', title: cap(m[1].trim()) };
  m = q.match(/^person:?\s+(.+)$/i);
  if (m) {
    const [first, ...rest] = m[1].trim().split(/\s+/);
    return { kind: 'person', first, last: rest.join(' ') };
  }
  m = q.match(/^timer\s+(\d+\s*[hms](?:\s*\d+\s*[hms])*)\s*(.*)$/i);
  if (m) {
    const ms = parseSpan(m[1]);
    if (ms) return { kind: 'timer', ms, label: m[1].replace(/\s+/g, ''), title: m[2] ? cap(m[2].trim()) : '' };
  }
  return null;
}

/** A free-text title as a note file name: "Meeting notes" → "meeting_notes". */
export const nameFromTitle = (title: string) =>
  title
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '');
