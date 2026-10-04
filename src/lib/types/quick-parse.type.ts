export type Phrase =
  | { kind: 'task'; title: string }
  | { kind: 'note'; title: string }
  | { kind: 'person'; first: string; last: string }
  | { kind: 'timer'; ms: number; label: string; title: string };
