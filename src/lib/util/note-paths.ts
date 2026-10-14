// Labels and links for the notes section: breadcrumbs and folder summaries.
import type { Crumb } from '#lib/types/breadcrumb.type.ts';
import { plural } from './plural.ts';
import { routes } from './routes.ts';

/** The name the notes root goes by in the explorer, headings and breadcrumbs. */
export const NOTES_ROOT_NAME = 'notes';

/** What deleting a note does, for its confirmation (the explorer and the note page). */
export const NOTE_DELETE_BODY = 'The file goes to the Trash. Tags and attachment links on it are removed.';

/** "notes / work / tauri": the root, then one link per folder segment. */
export function folderCrumbs(folder: string): Crumb[] {
  const crumbs: Crumb[] = [{ label: NOTES_ROOT_NAME, href: routes.notesFolder('') }];
  const segments = folder ? folder.split('/') : [];
  segments.forEach((segment, i) => {
    const path = segments.slice(0, i + 1).join('/');
    crumbs.push({ label: segment, href: routes.notesFolder(path) });
  });
  return crumbs;
}

/** "3 notes · 1 folder" under a folder heading. */
export function folderSummary({ notes, folders }: { notes: number; folders: number }): string {
  const noteCount = plural(notes, 'note');
  if (!folders) return noteCount;
  return `${noteCount} · ${plural(folders, 'folder')}`;
}

/** The delete dialog's account of what goes with a folder. */
export function folderContents({ notes, folders }: { notes: number; folders: number }): string {
  if (!notes && !folders) return 'The folder is empty.';
  const noteCount = plural(notes, 'note');
  const counts = folders ? `${noteCount} and ${plural(folders, 'folder')}` : noteCount;
  return `It contains ${counts}. Everything inside goes to the Trash.`;
}
