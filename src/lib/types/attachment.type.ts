export type Owner = 'task' | 'note' | 'comment' | 'person';

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
