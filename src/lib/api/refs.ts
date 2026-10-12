import type { NewRef, Ref, RefPatch } from '#lib/types/ref.type.ts';
import { call } from './core.ts';

export const refsAPI = {
  list: () => call<Ref[]>('refs_list'),
  create: (reference: NewRef) => call<Ref>('ref_create', { args: { reference } }),
  update: (input: { id: string; patch: RefPatch }) => call<Ref>('ref_update', { args: input }),
  remove: (id: string) => call<void>('ref_delete', { args: { id } }),
  /** Fetches the link's page title (Rust gives up after 5 s). */
  title: (url: string) => call<string | null>('ref_title', { args: { url } }),
  setTags: (input: { id: string; tags: string[] }) => call<string[]>('ref_tags_set', { args: input }),
};
