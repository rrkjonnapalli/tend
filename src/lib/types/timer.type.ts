export type Timer = { id: string; title: string; endsAt: number };

export type Duration = { ok: true; ms: number; label: string } | { ok: false; error: string };
