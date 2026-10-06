import { describe, expect, it } from 'vitest';
import { parsePhrase, parseSpan } from './quick-parse.ts';

describe('parsePhrase', () => {
  it('reads task, note and person phrases', () => {
    expect(parsePhrase('task: write release notes')).toEqual({ kind: 'task', title: 'Write release notes' });
    expect(parsePhrase('note Meeting notes')).toEqual({ kind: 'note', title: 'Meeting notes' });
    expect(parsePhrase('person: Ravi Kumar Rao')).toEqual({ kind: 'person', first: 'Ravi', last: 'Kumar Rao' });
  });

  it('reads timers with units', () => {
    expect(parsePhrase('timer 10m turn off the motor')).toEqual({ kind: 'timer', ms: 600_000, label: '10m', title: 'Turn off the motor' });
    expect(parsePhrase('timer 1h 30m')).toEqual({ kind: 'timer', ms: 5_400_000, label: '1h30m', title: '' });
  });

  it('ignores everything else', () => {
    expect(parsePhrase('go to notes')).toBeNull();
    expect(parsePhrase('timer soon')).toBeNull();
    expect(parsePhrase('task:')).toBeNull();
  });
});

describe('helpers', () => {
  it('parses spans', () => {
    expect(parseSpan('90s')).toBe(90_000);
    expect(parseSpan('5m30s')).toBe(330_000);
    expect(parseSpan('0m')).toBeNull();
  });
});
