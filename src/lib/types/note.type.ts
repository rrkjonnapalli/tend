/** `name` is the file name without .md (the title's slug); `title` is what the app shows. */
export type NoteRow = { id: string; folder: string; name: string; title: string; modifiedAt: number };
export type NoteTree = { folders: string[]; notes: NoteRow[] };
export type NoteDoc = NoteRow & { text: string; tags: string[] };
/** An item a scan left out because its name breaks the rule; `fix` is the path it can be renamed to. */
export type Skipped = { path: string; fix: string };
/** Something in the notes explorer: a note by id, or a folder by path. */
export type Item = { kind: 'note'; id: string } | { kind: 'folder'; path: string };
