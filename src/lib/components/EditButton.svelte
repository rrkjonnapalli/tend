<script lang="ts">
  // The pencil a list row shows on hover to edit its title in place. It may sit inside a link
  // row, so its click never follows the link, and it never starts a row drag.
  import Icon from './Icon.svelte';

  let { label, onedit }: { label: string; onedit: () => void } = $props();

  function onclick(e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    onedit();
  }
</script>

<button class="icon-btn sm edit on-hover" data-no-drag data-rename aria-label={label} title="{label} (F2)" {onclick}>
  <Icon name="edit" size={12} />
</button>

<style>
  .edit {
    flex: 0 0 auto;
  }
</style>
