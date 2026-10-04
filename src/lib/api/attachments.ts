import { call } from './core.ts';

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

export const attachmentsApi = {
  list: (owner: Owner, id: string) => call<AttachmentLink[]>('attachments_list', { owner, id }),
  add: (owner: Owner, id: string, paths: string[]) => call<void>('attachments_add', { owner, id, paths }),
  linkNote: (owner: Owner, id: string, note: string) => call<void>('attachment_link_note', { owner, id, note }),
  unlink: (link: string) => call<void>('attachment_unlink', { link }),
  open: (attachment: string) => call<void>('attachment_open', { attachment }),
  reveal: (attachment: string) => call<void>('attachment_reveal', { attachment }),
};
