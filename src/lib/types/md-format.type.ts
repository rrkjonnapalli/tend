/** A button of the phone's formatting bar (md-format.ts). */
export type MdFormat = 'heading' | 'bold' | 'italic' | 'list' | 'check' | 'link' | 'code';

/** The text that replaces [from, to], and where the selection lands after it. */
export type MdEdit = { from: number; to: number; insert: string; anchor: number; head: number };

/** A selection in a document, and the formatting to apply to it. */
export type MdFormatInput = { doc: string; from: number; to: number; kind: MdFormat };
