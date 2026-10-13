<script lang="ts">
  // Settings → Library → Dictionary: which dictionary a word is looked up in. Words already
  // fetched keep the entry they have, and say where it came from; Fetch again asks the new one.
  import Section from './Section.svelte';
  import ChoicePicker from './ChoicePicker.svelte';
  import { dictionarySources } from '#lib/config/library.config.ts';
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import type { DictionarySource } from '#lib/enums/dictionary.enum.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';

  const source = $derived(settings.get<DictionarySource>(settingKeys.dictionarySource, settingDefaults.dictionarySource));
  const about = $derived(dictionarySources.find((s) => s.id === source)?.about ?? '');
</script>

<Section title="Dictionary" indent={false}>
  <div class="pick">
    <ChoicePicker value={source} choices={dictionarySources} what="Dictionary" search={false} field onpick={(id) => settings.set(settingKeys.dictionarySource, id)} />
    <span class="caption muted">{about} Words you already looked up keep their entry until you fetch them again.</span>
  </div>
</Section>

<style>
  .pick {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 8px;
    padding-top: 4px;
  }
</style>
