<script lang="ts">
  import { markTarget, openComposer } from '../actions';
  import { app } from '../state.svelte';
</script>

{#if app.selection.length}
  <div class="selection-bar">
    <strong>{app.selection.length}</strong> line(s) selected
    <button class="primary" onclick={() => markTarget('accepted')}>Accept <kbd>a</kbd></button>
    <button onclick={() => markTarget('deferred')}>Later <kbd>d</kbd></button>
    <button onclick={() => markTarget(null)}>Reset <kbd>u</kbd></button>
    <button onclick={() => openComposer()}>Comment / ask <kbd>c</kbd></button>
    <button class="ghost" onclick={() => (app.selection = [])}>Clear <kbd>esc</kbd></button>
  </div>
{/if}
