<script lang="ts">
  import { app } from '../state.svelte';
  import { stepEntries } from '../step-labels';
  import AiChanges from './AiChanges.svelte';
  import LaterStep from './LaterStep.svelte';
  import QueueGroup from './QueueGroup.svelte';
  import QueueItem from './QueueItem.svelte';

  const entries = $derived(stepEntries(app.snap?.chunks ?? []));
  const showLater = $derived(app.hasLater || app.view?.kind === 'later');
</script>

{#if app.snap?.planning}
  <div class="planning"><span class="spinner"></span> AI is planning the review order…<br /><span class="muted">You can already start; marks are kept.</span></div>
{/if}
<AiChanges />
<ol class="queue">
  {#each entries as e (e.kind === 'step' ? e.chunk.id : `${e.label}|${e.title}`)}
    {#if e.kind === 'step'}<QueueItem chunk={e.chunk} label={e.label} />{:else}<QueueGroup title={e.title} label={e.label} steps={e.steps} />{/if}
  {/each}
  {#if showLater}<LaterStep label={String(entries.length + 1)} />{/if}
</ol>
