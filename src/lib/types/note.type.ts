/** `name` is the file name without .md, which the app shows; the title lives in the file's heading. */
export type NoteRow = { id: string; folder: string; name: string; modifiedAt: number };
export type NoteTree = { folders: string[]; notes: NoteRow[] };
export type NoteDoc = NoteRow & { text: string; tags: string[] };
/** An item a scan left out because its name breaks the rule; `fix` is the path it can be renamed to. */
export type Skipped = { path: string; fix: string };
/** Something in the notes explorer: a note by id, or a folder by path. */
export type Item = { kind: 'note'; id: string } | { kind: 'folder'; path: string };
