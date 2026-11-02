<script lang="ts">
  import { markKeys, selectLine } from '../actions';
  import type { Cell } from '../rows';
  import type { MarkStatus } from '../types';
  import { drag } from '../drag.svelte';

  let { cell, oldNo, newNo, status, unified }: { cell: Cell; oldNo: number | null; newNo: number | null; status: MarkStatus | undefined; unified: boolean } = $props();


  function down(e: MouseEvent) {
    if (!cell.key || e.button !== 0) return;
    e.preventDefault();
    selectLine(cell.key, e.shiftKey ? 'extend' : e.metaKey || e.ctrlKey ? 'toggle' : 'set');
    drag.active = !e.metaKey && !e.ctrlKey;
  }

  /** The status dot toggles this one line between accepted and pending. */
  function toggle(e: MouseEvent) {
    e.stopPropagation();
    e.preventDefault();
    if (cell.key) markKeys([cell.key], status === 'accepted' ? null : 'accepted', false);
  }

  function enter() {
    if (drag.active && cell.key) selectLine(cell.key, 'extend');
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<span class="gutter" class:selectable={!!cell.key} onmousedown={down} onmouseenter={enter}>
  {#if unified}
    <span class="no">{oldNo ?? ''}</span><span class="no">{newNo ?? ''}</span>
  {:else}
    <span class="no">{cell.no || ''}</span>
  {/if}
  {#if cell.key}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <span class="mk toggle-mk" role="button" tabindex="-1" title={status === 'accepted' ? 'Accepted — click to reset' : 'Click to accept this line'} onmousedown={(e) => e.stopPropagation()} onclick={toggle}>{status === 'accepted' ? '✓' : status === 'deferred' ? '⏸' : '•'}</span>
  {:else}
    <span class="mk"></span>
  {/if}
</span>
