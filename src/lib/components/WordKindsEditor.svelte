<script lang="ts">
  // Settings → Library → Word kinds: what the Words list groups by. Tend's own kinds can be renamed
  // and moved, not deleted; a kind some word has can't be deleted either.
  import ColorListEditor from './ColorListEditor.svelte';
  import { wordsAPI } from '#lib/api/words.ts';
  import type { ColorItem, ColorListOps } from '#lib/types/color-list.type.ts';
  import { words } from '#lib/stores/words.svelte.ts';

  words.load();

  const items = $derived(
    words.kinds.map((k): ColorItem => ({ key: k.value, name: k.label, meta: k.value, removable: !k.isDefault })),
  );

  const intro =
    'Words are grouped by kind. New kinds aren’t looked up in the dictionary; word, idiom and phrase are.';

  const ops: ColorListOps = {
    add: ({ name, value }) => wordsAPI.addKind({ label: name, value }),
    rename: ({ key, name }) => wordsAPI.renameKind({ value: key, label: name }),
    reorder: (order) => wordsAPI.reorderKinds(order),
    remove: (value) => wordsAPI.removeKind(value),
    reload: () => words.load(),
  };
</script>

<ColorListEditor title="Word kinds" {intro} what="kind" {items} {ops} />
