import type { SourceType } from '#lib/enums/link.enum.ts';

/** The record attachments hang off. */
export type Owner = SourceType;

export type AttachmentLink = {
  id: string;
  kind: 'file' | 'note';
  attachmentId: string | null;
  noteId: string | null;
  name: string;
  mimeType: string | null;
  sizeBytes: number | null;
  folder: string | null;
  createdAt: number;
};

export type OwnerRef = { owner: Owner; id: string };
