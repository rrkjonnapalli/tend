export type NoteRow = { id: string; folder: string; name: string; title: string; modifiedAt: number };
export type NoteTree = { folders: string[]; notes: NoteRow[] };
export type NoteDoc = NoteRow & { text: string; tags: string[] };
/** Something in the notes explorer: a note by id, or a folder by path. */
export type Item = { kind: 'note'; id: string } | { kind: 'folder'; path: string };
