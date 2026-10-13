import type { EnumStatus } from '#lib/enums/status.enum.ts';
import type { TagRow } from '#lib/types/tag.type.ts';
import { call } from './core.ts';

export const tagsAPI = {
  /** The names of the active tags. */
  all: () => call<string[]>('tags_all'),
  /** Every tag, blocked ones too, by name, with how many things carry each. */
  overview: () => call<TagRow[]>('tags_overview'),
  /** Blocking a tag takes it off everything that carries it. */
  setStatus: (input: { name: string; status: EnumStatus }) => call<void>('tag_set_status', { args: input }),
  /** Deleting a tag takes it out of the pool and off everything that carries it. */
  remove: (name: string) => call<void>('tag_delete', { args: { name } }),
  setForNote: (input: { id: string; tags: string[] }) => call<string[]>('note_tags_set', { args: input }),
};
