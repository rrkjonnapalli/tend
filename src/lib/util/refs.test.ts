import { describe, expect, it } from 'vitest';
import { REF_KIND } from '#lib/enums/ref.enum.ts';
import type { Ref } from '#lib/types/ref.type.ts';
import { isUrl, kindOf, openableUrl, refTitle, shownUrl, siteOf, urlKey } from './refs.ts';

describe('refs', () => {
  it('tells links from text', () => {
    expect(isUrl('https://svelte.dev/blog/runes')).toBe(true);
    expect(isUrl('svelte.dev')).toBe(true);
    expect(isUrl('Perfect Days')).toBe(false);
    expect(isUrl('e.g. this')).toBe(false);
    expect(kindOf(' paulgraham.com/greatwork.html ')).toBe(REF_KIND.link);
  });

  it('keys a link like Rust does', () => {
    for (const same of ['https://Svelte.dev/blog/runes/', 'http://svelte.dev/blog/runes', '  SVELTE.DEV/BLOG/RUNES  ']) {
      expect(urlKey(same)).toBe('svelte.dev/blog/runes');
    }
    expect(urlKey('www.svelte.dev')).not.toBe(urlKey('svelte.dev'));
  });

  it('shows and opens links', () => {
    expect(shownUrl('https://www.youtube.com/watch?v=x')).toBe('youtube.com/watch?v=x');
    expect(siteOf('https://www.youtube.com/watch?v=x')).toBe('youtube.com');
    expect(openableUrl('svelte.dev')).toBe('https://svelte.dev');
    expect(openableUrl('http://a.dev')).toBe('http://a.dev');
  });

  it('falls back to the link or the text for a missing title', () => {
    const base: Ref = { id: 'r', kind: REF_KIND.link, value: 'https://www.a.dev/x', title: null, why: '', urlKey: 'a.dev/x', tags: [], createdAt: 0, updatedAt: 0 };
    expect(refTitle(base)).toBe('a.dev/x');
    expect(refTitle({ ...base, kind: REF_KIND.text, value: 'Severance' })).toBe('Severance');
    expect(refTitle({ ...base, title: 'A' })).toBe('A');
  });
});
