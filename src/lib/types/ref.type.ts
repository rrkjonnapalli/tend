import type { RefKind } from '#lib/enums/ref.enum.ts';

/** A saved reference, as `refs_list` returns it. */
export type Ref = {
  id: string;
  kind: RefKind;
  /** The URL or the text as entered. */
  value: string;
  /** Fetched or typed; `null` shows the value. */
  title: string | null;
  why: string;
  /** A link's normalised URL; `null` for text. */
  urlKey: string | null;
  tags: string[];
  createdAt: number;
  updatedAt: number;
};

export type NewRef = { kind: RefKind; value: string; title: string | null; why: string };

/** The open reference popup: `editing` null adds a new one, with `filled` (text from Home) in. */
export type RefFormState = { editing: Ref | null; filled?: string };

/** Fields an edit can change; `title: null` clears it. */
export type RefPatch = Partial<NewRef>;
