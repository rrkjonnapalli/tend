import { describe, expect, it } from 'vitest';
import { keyAt, keyAfter, keyBetween } from './order-key.ts';

describe('keyBetween', () => {
  it('makes a key between two neighbours, and at either open end', () => {
    const first = keyBetween(null, null);
    const next = keyBetween(first, null);
    const mid = keyBetween(first, next);
    expect(first < mid && mid < next).toBe(true);
    expect(keyBetween(null, first) < first).toBe(true);
  });

  it('keeps keys short when items are always added at the top or the bottom', () => {
    let top = keyBetween(null, null);
    let bottom = top;
    for (let i = 0; i < 10_000; i++) {
      top = keyBetween(null, top);
      bottom = keyBetween(bottom, null);
    }
    expect(top.length).toBeLessThan(8);
    expect(bottom.length).toBeLessThan(8);
  });

  it('keeps order over many inserts between the same neighbours', () => {
    let lo = keyBetween(null, null);
    const hi = keyBetween(lo, null);
    for (let i = 0; i < 200; i++) {
      const k = keyBetween(lo, hi);
      expect(k > lo && k < hi).toBe(true);
      lo = k;
    }
  });

  it('rejects neighbours in the wrong order', () => {
    const a = keyBetween(null, null);
    expect(() => keyBetween(keyBetween(a, null), a)).toThrow();
  });
});

describe('keyAt and keyAfter', () => {
  const [b, d, f] = ['a1', 'a3', 'a5'];

  it('places an item at an index', () => {
    const keys = [b, d, f];
    const k = keyAt(keys, 1);
    expect(k > b && k < d).toBe(true);
    expect(keyAt(keys, 0) < b).toBe(true);
    expect(keyAt(keys, 3) > f).toBe(true);
  });

  it('drops between tied keys without failing, right after the ties', () => {
    const k = keyAt([b, d, d, f], 2);
    expect(k > d && k < f).toBe(true);
    expect(keyAfter(d, [d, d]) > d).toBe(true);
  });
});
