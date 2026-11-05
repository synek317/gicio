<script lang="ts">
  import { run } from '../actions';
  import { api } from '../api';
  import { touchesCSharp, touchesRust } from '../project-tools';
  import { reviewWithAi, runTool } from '../review-actions';
  import { app } from '../state.svelte';
  import FindingCard from './FindingCard.svelte';

  let showClosed = $state(false);
  const all = $derived(app.snap?.findings ?? []);
  const open = $derived(all.filter((f) => f.status === 'open'));
  /** Changes beyond the task may well be wanted, so undoing them is always a choice of its own. */
  const fixable = $derived(open.filter((f) => f.source !== 'intent'));
  const shown = $derived((showClosed ? all : all.filter((f) => f.status === 'open' || f.status === 'fixing')).toSorted((a, b) => a.path.localeCompare(b.path) || a.line - b.line));
  const running = $derived(app.jobs.some((j) => (j.kind === 'aiReview' || j.kind === 'intentCheck' || j.title === 'cargo clippy' || j.title === 'dotnet build') && j.status === 'running'));
  const csharp = $derived(touchesCSharp(app.snap?.files ?? []));
  const rust = $derived(touchesRust(app.snap?.files ?? []) || !csharp);
  const chunkId = $derived(app.view?.kind === 'chunk' ? app.view.id : null);
</script>

<div class="threads-body findings-panel">
  <div class="row wrap">
    <button class="small" disabled={!chunkId} onclick={() => chunkId && reviewWithAi({ kind: 'chunk', id: chunkId })}>AI review this step</button>
    <button class="small" onclick={() => reviewWithAi({ kind: 'all' })}>AI review everything</button>
    {#if rust}<button class="small" onclick={() => runTool('clippy')}>clippy</button>{/if}
    {#if csharp}<button class="small" onclick={() => runTool('dotnetBuild')}>dotnet build</button>{/if}
    {#if running}<span class="spinner"></span>{/if}
  </div>
  {#if open.length}
    <div class="row">
      {#if fixable.length}<button class="primary small" onclick={() => run(api.fixFindings(fixable.map((f) => f.id)))}>Fix all open ({fixable.length})</button>{/if}
      <button class="ghost small" onclick={() => run(api.setFindingsStatus(open.map((f) => f.id), 'dismissed'))}>Dismiss all</button>
      <span class="spacer"></span>
      <label class="toggle"><input type="checkbox" bind:checked={showClosed} /> closed</label>
    </div>
  {/if}
  {#each shown as f (f.id)}
    <div class="card"><FindingCard finding={f} /></div>
  {:else}
    <p class="muted small hint">No findings. Run an AI review (whole change, this step, or select lines and choose “AI review”) or a linter (clippy, dotnet build). Findings appear here and next to the code, with one-click fixes.</p>
  {/each}
</div>
