<script lang="ts">
  import { untrack } from 'svelte';
  import { api } from '../api';
  import { renderMarkdown } from '../markdown';
  import { app } from '../state.svelte';
  import JobLog from './JobLog.svelte';

  const s = $derived(app.snap!);
  const job = $derived(app.jobs.findLast((j) => j.kind === 'summary' && j.status === 'running'));
  // The summary is the first thing to read; once a conversation is going it folds away.
  let open = $state(untrack(() => !app.snap?.chat.length));

  function generate(e: MouseEvent) {
    e.preventDefault();
    open = true;
    api.summarize().catch((err) => app.toast(String(err), 'error'));
  }
</script>

<details class="chat-summary" bind:open>
  <summary class="row">
    <span class="chat-summary-title">Summary of the change</span>
    <span class="spacer"></span>
    {#if s.summaryBusy}
      <span class="spinner"></span>
      {#if job}<button class="ghost small" onclick={(e) => { e.preventDefault(); api.cancelJob(job.id); }}>Stop</button>{/if}
    {:else}
      <button class="small" class:primary={!s.summary} onclick={generate}>{s.summary ? 'Regenerate' : 'Generate'}</button>
    {/if}
  </summary>
  {#if s.summaryBusy && job}<JobLog {job} compact />{/if}
  {#if s.summary}
    <div class="md">{@html renderMarkdown(s.summary.body)}</div>
  {:else if !s.summaryBusy}
    <p class="muted small">What the change does, how, its risks and where to look closely, written by the AI from the whole diff.</p>
  {/if}
</details>
