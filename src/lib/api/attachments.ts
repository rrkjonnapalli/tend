import { config } from '#lib/config/app.config.ts';
import type { AttachmentLink, OwnerRef } from '#lib/types/attachment.type.ts';
import { call } from './core.ts';

export const attachmentsAPI = {
  list: (ref: OwnerRef) => call<AttachmentLink[]>('attachments_list', { args: ref }),
  /** Copies the files into the vault, so large files get the long deadline. */
  add: (input: OwnerRef & { paths: string[] }) =>
    call<void>('attachments_add', { args: input, timeoutMs: config.slowCallTimeoutMs }),
  linkNote: (input: OwnerRef & { note: string }) => call<void>('attachment_link_note', { args: input }),
  unlink: (link: string) => call<void>('attachment_unlink', { args: { link } }),
  open: (attachment: string) => call<void>('attachment_open', { args: { attachment } }),
  reveal: (attachment: string) => call<void>('attachment_reveal', { args: { attachment } }),
};
