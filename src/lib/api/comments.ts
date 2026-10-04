import { call } from './core.ts';

export type Comment = { id: string; body: string; createdAt: number; editedAt: number | null };

export const commentsApi = {
  list: (task: string) => call<Comment[]>('comments_list', { task }),
  add: (task: string, body: string) => call<void>('comment_add', { task, body }),
  edit: (id: string, body: string) => call<void>('comment_edit', { id, body }),
};
