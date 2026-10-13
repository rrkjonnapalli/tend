<script lang="ts">
  // The push-pin at the top middle of a snap's paper (its parent must be positioned): red, brass
  // and a little bigger when the snap is pinned to the top, small on Home's mini board.
  let { top = false, mini = false }: { top?: boolean; mini?: boolean } = $props();
</script>

<span class="pin" class:top class:mini aria-hidden="true"></span>

<style>
  .pin {
    --pin-size: 14px;
    position: absolute;
    top: 8px;
    left: calc(50% - var(--pin-size) / 2);
    width: var(--pin-size);
    height: var(--pin-size);
    border-radius: 50%;
    background: var(--pin-red);
    box-shadow: var(--pin-shadow);
  }

  .top {
    --pin-size: 16px;
    background: var(--pin-brass);
  }

  .mini {
    --pin-size: 11px;
    top: 7px;
  }
</style>
