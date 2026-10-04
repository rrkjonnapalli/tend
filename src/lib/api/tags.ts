import { call } from './core.ts';

export const tagsApi = {
  all: () => call<string[]>('tags_all'),
  setForNote: (id: string, tags: string[]) => call<string[]>('note_tags_set', { id, tags }),
};
