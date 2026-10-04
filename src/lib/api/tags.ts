import { call } from './core.ts';

export const tagsAPI = {
  all: () => call<string[]>('tags_all'),
  setForNote: (input: { id: string; tags: string[] }) => call<string[]>('note_tags_set', { args: input }),
};
