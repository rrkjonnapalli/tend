import { describe, expect, it } from 'vitest';
import { keyAt, keyAfter, keyBetween } from './order-key.ts';

describe('keyBetween', () => {
  it('makes a key between two neighbours', () => {
    const k = keyBetween('a', 'c');
    expect(k > 'a' && k < 'c').toBe(true);
  });

  it('handles open ends', () => {
    const first = keyBetween(null, null);
    expect(keyBetween(null, first) < first).toBe(true);
    expect(keyBetween(first, null) > first).toBe(true);
  });

  it('works between adjacent digits by going deeper', () => {
    const k = keyBetween('a', 'b');
    expect(k > 'a' && k < 'b').toBe(true);
  });

  it('keeps order over many inserts between the same neighbours', () => {
    let lo = keyBetween(null, null);
    const hi = keyBetween(lo, null);
    const made: string[] = [];
    for (let i = 0; i < 200; i++) {
      const k = keyBetween(lo, hi);
      expect(k > lo && k < hi).toBe(true);
      made.push(k);
      lo = k;
    }
    expect([...made].sort()).toEqual(made);
  });

  it('keeps order when always inserting at the front', () => {
    let hi = keyBetween(null, null);
    for (let i = 0; i < 200; i++) {
      const k = keyBetween(null, hi);
      expect(k < hi).toBe(true);
      hi = k;
    }
  });

  it('rejects neighbours in the wrong order', () => {
    expect(() => keyBetween('b', 'a')).toThrow();
  });
});

describe('keyAt', () => {
  it('places an item at an index', () => {
    const keys = ['b', 'd', 'f'];
    const k = keyAt(keys, 1);
    expect(k > 'b' && k < 'd').toBe(true);
    expect(keyAt(keys, 0) < 'b').toBe(true);
    expect(keyAt(keys, 3) > 'f').toBe(true);
  });
});

describe('keyAfter', () => {
  it('drops between tied keys without failing, right after the ties', () => {
    const keys = ['b', 'd', 'd', 'f'];
    const k = keyAt(keys, 2);
    expect(k > 'd' && k < 'f').toBe(true);
    expect(keyAfter('d', ['d', 'd'])).toBeTruthy();
    expect(keyAfter('d', ['d', 'd']) > 'd').toBe(true);
  });
});
