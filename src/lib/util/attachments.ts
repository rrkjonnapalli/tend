// Labels for attachment rows.
import type { AttachmentLink } from '#lib/types/attachment.type.ts';
import type { IconName } from '#lib/types/icon.type.ts';
import { bytesLabel } from './bytes.ts';
import { iso, shortDate } from './dates.ts';

/** "report.pdf" → "PDF"; a name with no dot is a "FILE". */
export function extension(name: string): string {
  const dot = name.lastIndexOf('.');
  if (dot === -1) return 'FILE';
  return name.slice(dot + 1).toUpperCase();
}

export function attachmentIcon(l: AttachmentLink): IconName {
  if (l.kind === 'note') return 'note';
  const isImage = l.mimeType?.startsWith('image/') ?? false;
  return isImage ? 'image' : 'file';
}

/** "note · projects" or "PDF · 240 KB · 9 Oct". */
export function attachmentMeta(l: AttachmentLink, now = new Date()): string {
  if (l.kind === 'note') return `note · ${l.folder || 'notes'}`;
  const size = bytesLabel(l.sizeBytes ?? 0);
  const added = iso(new Date(l.createdAt));
  return `${extension(l.name)} · ${size} · ${shortDate(added, now)}`;
}
