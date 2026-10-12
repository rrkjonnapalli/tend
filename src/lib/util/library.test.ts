import { describe, expect, it } from 'vitest';
import { LIBRARY_AREA } from '#lib/enums/library.enum.ts';
import { REF_KIND } from '#lib/enums/ref.enum.ts';
import { PAPER } from '#lib/enums/snap.enum.ts';
import type { Ref } from '#lib/types/ref.type.ts';
import type { Snap } from '#lib/types/snap.type.ts';
import { guessArea, newest, refHit, snapHit } from './library.ts';

const snap = (over: Partial<Snap>): Snap => ({
  id: 's1',
  stackId: null,
  text: '',
  paper: PAPER.scrap,
  color: null,
  pinned: false,
  tags: [],
  createdAt: 0,
  updatedAt: 0,
  ...over,
});

const ref = (over: Partial<Ref>): Ref => ({
  id: 'r1',
  kind: REF_KIND.text,
  value: '',
  title: null,
  why: '',
  urlKey: null,
  tags: [],
  createdAt: 0,
  updatedAt: 0,
  ...over,
});

describe('guessArea', () => {
  it('has no guess for nothing', () => {
    expect(guessArea('  \n ')).toBeNull();
  });

  it('sends a link to References', () => {
    expect(guessArea(' https://svelte.dev/blog/runes ')?.area).toBe(LIBRARY_AREA.refs);
    expect(guessArea('paulgraham.com/greatwork.html')?.area).toBe(LIBRARY_AREA.refs);
  });

  it('sends one short line to Words', () => {
    expect(guessArea('ephemeral')?.area).toBe(LIBRARY_AREA.words);
    expect(guessArea('bite the bullet now')?.area).toBe(LIBRARY_AREA.words);
  });

  it('sends anything longer to Snaps', () => {
    expect(guessArea('one two three four five')?.area).toBe(LIBRARY_AREA.snaps);
    expect(guessArea('two\nlines')?.area).toBe(LIBRARY_AREA.snaps);
    expect(guessArea('pneumonoultramicroscopicsilicovolcanoconiosis')?.area).toBe(LIBRARY_AREA.snaps);
  });
});

describe('newest', () => {
  it('takes the last added first and leaves the list as it was', () => {
    const items = [{ createdAt: 1 }, { createdAt: 3 }, { createdAt: 2 }];
    expect(newest(items, 2)).toEqual([{ createdAt: 3 }, { createdAt: 2 }]);
    expect(items[0].createdAt).toBe(1);
    expect(newest([], 4)).toEqual([]);
  });
});

describe('hits', () => {
  it('shows a snap by its first line and opens its stack', () => {
    const now = new Date(10_000);
    const hit = snapHit(snap({ text: '\nfirst\nsecond', tags: ['poem', 'x'], stackId: 'k1', updatedAt: 10_000 }), now);
    expect(hit.title).toBe('first');
    expect(hit.sub).toBe('#poem +1 · now');
    expect(hit.href).toBe('/library/snaps?s=k1');
    expect(snapHit(snap({ text: 'loose', updatedAt: 10_000 }), now).sub).toBe('now');
  });

  it('shows a link by its address and text as text', () => {
    const link = refHit(ref({ kind: REF_KIND.link, value: 'https://www.svelte.dev/blog' }));
    expect(link).toMatchObject({ title: 'svelte.dev/blog', sub: 'svelte.dev/blog', mono: true });
    const text = refHit(ref({ value: 'Perfect Days' }));
    expect(text).toMatchObject({ title: 'Perfect Days', sub: 'text', mono: false });
  });
});
