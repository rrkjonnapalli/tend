import type { Comment } from '#lib/types/comment.type.ts';
import { call } from './core.ts';

export const commentsAPI = {
  list: (task: string) => call<Comment[]>('comments_list', { args: { task } }),
  add: (input: { task: string; body: string }) => call<void>('comment_add', { args: input }),
  edit: (input: { id: string; body: string }) => call<void>('comment_edit', { args: input }),
};
