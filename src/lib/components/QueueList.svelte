<script lang="ts">
  import { progressState } from '../progress';
  import { app } from '../state.svelte';
  import QueueItem from './QueueItem.svelte';

  const items = $derived((app.snap?.chunks ?? []).map((chunk, index) => ({ chunk, index, done: progressState(app.chunkProgress(chunk)) === 'done' })));
  const todo = $derived(items.filter((i) => !i.done));
  const done = $derived(items.filter((i) => i.done));
  let showDone = $state(true);
</script>

{#if app.snap?.planning}
  <div class="planning"><span class="spinner"></span> AI is planning the review order…<br /><span class="muted">You can already start; marks are kept.</span></div>
{/if}
<h3 class="qhead">To review <span class="muted">({todo.length})</span></h3>
<ol class="queue">
  {#each todo as i (i.chunk.id)}<QueueItem chunk={i.chunk} index={i.index} />{:else}<li class="muted small qempty">Nothing left 🎉</li>{/each}
</ol>
<button class="qhead toggle-head" onclick={() => (showDone = !showDone)}>{showDone ? '▾' : '▸'} Done <span class="muted">({done.length})</span></button>
{#if showDone}
  <ol class="queue done-list">
    {#each done as i (i.chunk.id)}<QueueItem chunk={i.chunk} index={i.index} />{:else}<li class="muted small qempty">Nothing accepted yet.</li>{/each}
  </ol>
{/if}
