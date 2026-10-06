// Phrases the ⌘K palette turns into things: "task: …", "note: …", "person: …", "timer 10m …".
import type { Phrase } from '#lib/types/quick-parse.type.ts';

const UNIT_MS: Record<string, number> = { h: 3_600_000, m: 60_000, s: 1000 };

const capitalize = (text: string) => text.charAt(0).toUpperCase() + text.slice(1);
const withoutSpaces = (text: string) => text.replace(/\s/g, '');

/** "1h 30m", "10m", "90s", "5m30s" → milliseconds. */
export function parseSpan(text: string): number | null {
  const parts = [...text.matchAll(/(\d+)\s*([hms])/gi)];
  if (!parts.length) return null;
  const matched = parts.map((p) => p[0]).join('');
  const coversAll = withoutSpaces(matched) === withoutSpaces(text);
  if (!coversAll) return null;

  let ms = 0;
  for (const [, amount, unit] of parts) ms += Number(amount) * UNIT_MS[unit.toLowerCase()];
  return ms > 0 ? ms : null;
}

function parseTask(q: string): Phrase | null {
  const m = q.match(/^task:?\s+(.+)$/i);
  if (!m) return null;
  return { kind: 'task', title: capitalize(m[1].trim()) };
}

function parseNote(q: string): Phrase | null {
  const m = q.match(/^note:?\s+(.+)$/i);
  if (!m) return null;
  return { kind: 'note', title: capitalize(m[1].trim()) };
}

function parsePerson(q: string): Phrase | null {
  const m = q.match(/^person:?\s+(.+)$/i);
  if (!m) return null;
  const [first, ...rest] = m[1].trim().split(/\s+/);
  return { kind: 'person', first, last: rest.join(' ') };
}

function parseTimer(q: string): Phrase | null {
  const m = q.match(/^timer\s+(\d+\s*[hms](?:\s*\d+\s*[hms])*)\s*(.*)$/i);
  if (!m) return null;
  const [, span, title] = m;
  const ms = parseSpan(span);
  if (!ms) return null;
  return { kind: 'timer', ms, label: withoutSpaces(span), title: capitalize(title.trim()) };
}

export function parsePhrase(input: string): Phrase | null {
  const q = input.trim();
  return parseTask(q) ?? parseNote(q) ?? parsePerson(q) ?? parseTimer(q);
}
