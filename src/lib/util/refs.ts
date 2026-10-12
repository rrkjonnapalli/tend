// References: telling a link from text, the duplicate key (same rule as Rust `refs::url_key`) and
// how a link is shown.
import { REF_KIND, type RefKind } from '#lib/enums/ref.enum.ts';
import type { Ref } from '#lib/types/ref.type.ts';

const SCHEME = /^https?:\/\//i;
const WITH_SCHEME = /^https?:\/\/\S+$/i;
/** A bare domain with an optional path: `svelte.dev/blog/runes`. */
const BARE_DOMAIN = /^[a-z0-9-]+(\.[a-z0-9-]+)+(\/\S*)?$/i;

export function isUrl(text: string): boolean {
  const t = text.trim();
  return WITH_SCHEME.test(t) || BARE_DOMAIN.test(t);
}

/** The kind typed text suggests: a URL is a link, anything else text. */
export const kindOf = (text: string): RefKind => (isUrl(text) ? REF_KIND.link : REF_KIND.text);

/** A link's duplicate key: scheme, one trailing slash and letter case ignored. */
export function urlKey(url: string): string {
  const key = url.trim().toLowerCase().replace(SCHEME, '');
  return key.endsWith('/') ? key.slice(0, -1) : key;
}

/** The URL to open: as typed when it has a scheme, else https. */
export const openableUrl = (url: string) => (SCHEME.test(url.trim()) ? url.trim() : `https://${url.trim()}`);

/** A link as the list shows it: no scheme, no `www.`. */
export const shownUrl = (url: string) => url.trim().replace(SCHEME, '').replace(/^www\./i, '');

/** The site a link is on: `svelte.dev`. */
export const siteOf = (url: string) => shownUrl(url).split(/[/?#]/)[0];

/** What the list calls a reference: its title, else the link as shown or the text. */
export function refTitle(ref: Ref): string {
  if (ref.title) return ref.title;
  return ref.kind === REF_KIND.link ? shownUrl(ref.value) : ref.value;
}
