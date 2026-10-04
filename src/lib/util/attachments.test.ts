import { expect, it } from 'vitest';
import { attachmentIcon, attachmentMeta, extension } from './attachments.ts';
import type { AttachmentLink } from '#lib/types/attachment.type.ts';

const NOW = new Date(2026, 9, 3, 12, 0);

const file: AttachmentLink = {
  id: 'l1',
  kind: 'file',
  attachmentId: 'a1',
  noteId: null,
  name: 'report.pdf',
  mimeType: 'application/pdf',
  sizeBytes: 245_760,
  folder: null,
  createdAt: new Date(2026, 9, 1).getTime(),
};

it('reads the extension', () => {
  expect(extension('report.final.pdf')).toBe('PDF');
  expect(extension('README')).toBe('FILE');
});

it('picks the row icon', () => {
  expect(attachmentIcon(file)).toBe('file');
  expect(attachmentIcon({ ...file, mimeType: 'image/png' })).toBe('image');
  expect(attachmentIcon({ ...file, kind: 'note' })).toBe('note');
});

it('describes the attachment', () => {
  expect(attachmentMeta(file, NOW)).toMatch(/^PDF · 240 KB · /);
  expect(attachmentMeta({ ...file, kind: 'note', folder: '' })).toBe('note · notes');
  expect(attachmentMeta({ ...file, kind: 'note', folder: 'work' })).toBe('note · work');
});
