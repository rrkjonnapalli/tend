// Library Home: where typed text probably belongs, the newest of a list, and each kind of item
// as Home lists it.
import { wordGuessLimit } from '#lib/config/library.config.ts';
import { LIBRARY_AREA } from '#lib/enums/library.enum.ts';
import { REF_KIND } from '#lib/enums/ref.enum.ts';
import type { AreaGuess, LibraryHit } from '#lib/types/library.type.ts';
import type { Ref } from '#lib/types/ref.type.ts';
import type { Snap } from '#lib/types/snap.type.ts';
import type { Word } from '#lib/types/word.type.ts';
import { ago } from './dates.ts';
import { isUrl, refTitle, shownUrl } from './refs.ts';
import { routes } from './routes.ts';
import { tagLabel } from './snaps.ts';
import { firstLine } from './words.ts';

/** What a text reference shows in place of a link. */
const TEXT_REF = 'text';

/** A URL is a reference; one short line a word or phrase; anything else a snap. Empty: none. */
export function guessArea(text: string): AreaGuess | null {
  const t = text.trim();
  if (!t) return null;
  if (isUrl(t)) return { area: LIBRARY_AREA.refs, why: 'It’s a link.' };
  const oneLine = !t.includes('\n');
  const short = t.split(/\s+/).length <= wordGuessLimit.words && t.length <= wordGuessLimit.chars;
  if (oneLine && short) return { area: LIBRARY_AREA.words, why: 'Short, so probably a word or phrase.' };
  return { area: LIBRARY_AREA.snaps, why: 'Reads like something you wrote.' };
}

/** The `count` items added last, newest first. */
export function newest<T extends { createdAt: number }>(items: T[], count: number): T[] {
  return [...items].sort((a, b) => b.createdAt - a.createdAt).slice(0, count);
}

export function wordHit(word: Word): LibraryHit {
  return { id: word.id, title: word.title, sub: firstLine(word.meaning), href: routes.libraryWord(word.id) };
}

/** A snap by its first line, with its first tag and when it was last changed; opens its stack. */
export function snapHit(snap: Snap, now = new Date()): LibraryHit {
  const title = snap.text.trim().split('\n')[0];
  const sub = [tagLabel(snap.tags), ago(snap.updatedAt, now)].filter(Boolean).join(' · ');
  return { id: snap.id, title, sub, href: routes.libraryStack(snap.stackId), plain: true };
}

export function refHit(ref: Ref): LibraryHit {
  const isLink = ref.kind === REF_KIND.link;
  const sub = isLink ? shownUrl(ref.value) : TEXT_REF;
  return { id: ref.id, title: refTitle(ref), sub, href: routes.libraryArea(LIBRARY_AREA.refs), plain: true, mono: isLink };
}
