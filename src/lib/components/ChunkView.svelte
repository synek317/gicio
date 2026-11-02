<script lang="ts">
  import { acceptRestAndNext, markView, nextPendingChunk, openComposer } from '../actions';
  import { renderMarkdown } from '../markdown';
  import { resolveLater } from '../review-actions';
  import { hunksProgress } from '../progress';
  import { app } from '../state.svelte';
  import type { Hunk } from '../types';
  import FileBlock from './FileBlock.svelte';
  import ProgressBar from './ProgressBar.svelte';
  import SelectionPopover from './SelectionPopover.svelte';
  import { previewOff, previewOn } from '../preview';
  import { trackVisibleFiles, updateVisibleFiles } from '../files-nav';
  import { tick } from 'svelte';

  const v = $derived(app.view);
  const chunk = $derived(v?.kind === 'chunk' ? app.chunk(v.id) : undefined);
  const hunks = $derived(app.viewHunks);
  const prog = $derived(hunksProgress(hunks, app.marks));

  /** Consecutive hunks of one file form one block. */
  const groups = $derived.by(() => {
    const out: { path: string; hunks: Hunk[] }[] = [];
    for (const h of hunks) {
      const last = out[out.length - 1];
      if (last?.path === h.path) last.hunks.push(h);
      else out.push({ path: h.path, hunks: [h] });
    }
    return out;
  });

  let headH = $state(0);

  $effect(() => trackVisibleFiles());
  // Blocks appear, collapse or grow: recompute after the DOM settles.
  $effect(() => {
    void app.viewHunks;
    void app.collapsed;
    void app.expand;
    tick().then(updateVisibleFiles);
  });

  const title = $derived(chunk?.title ?? (v?.kind === 'later' ? 'Resolve issues left for later' : v?.kind === 'file' ? v.path : ''));
  const waiting = $derived(app.laterThreads.filter((t) => !t.busy).length);
  const resolving = $derived(app.jobs.some((j) => j.kind === 'review' && (j.status === 'running' || j.status === 'queued')));
</script>

<div class="chunk-view" style="--head-h:{headH}px">
  <header class="chunk-head" bind:offsetHeight={headH}>
    <div class="title-row">
      <h1>{#if chunk?.group}<span class="muted feature" title="The feature this step belongs to">{chunk.group} ›</span> {/if}{title}</h1>
      {#if chunk?.origin.kind === 'followUp'}<span class="badge follow">follow-up</span>{/if}
    </div>
    {#if chunk?.summary}
      <div class="summary md">{@html renderMarkdown(chunk.summary)}</div>
    {:else if v?.kind === 'later'}
      <p class="summary muted">Conversations you left for later are on the right: let the AI take them all at once, or resolve them yourself. Below is every hunk that still has lines you put off.</p>
      <div class="later-bar row">
        {#if resolving}<span class="spinner"></span> The AI is working through the issues…{:else if waiting}<strong>{waiting}</strong> conversation(s) waiting{:else}<span class="muted">No conversations waiting.</span>{/if}
        <span class="spacer"></span>
        <button class="primary" disabled={!waiting} onclick={resolveLater} title="Sends every conversation left for later to the AI as one task. Code changes land as a new step.">Let the AI resolve them</button>
      </div>
    {/if}
    {#if prog.total}
    <div class="head-actions">
      <ProgressBar p={prog} />
      <span class="muted" title="{prog.accepted} accepted · {prog.deferred} later · {prog.pending} pending">{prog.pending ? `${prog.pending} pending` : 'all reviewed'}</span>
      <span class="spacer"></span>
      {#if chunk}<button class="ghost" onclick={() => openComposer(false)} title="New conversation about this step">Ask</button>{/if}
      {#if prog.pending}
        <button onclick={() => markView('deferred')} title="Put everything still pending in this step off for later">Later</button>
        <button class="primary" onmouseenter={() => previewOn(app.orderedKeys)} onmouseleave={previewOff} onclick={acceptRestAndNext} title="Accept everything still pending and go to the next step (Shift+A)">Accept rest & next</button>
      {/if}
    </div>
    {/if}
  </header>

  {#if prog.total > 0 && prog.pending === 0}
    <div class="done-banner">
      Nothing pending here.
      <button class="primary" onclick={nextPendingChunk}>Next chunk (Space)</button>
    </div>
  {/if}

  {#each groups as g, i (g.path + i)}
    <FileBlock path={g.path} hunks={g.hunks} />
  {:else}
    <p class="empty muted">{v?.kind === 'later' ? 'No lines put off.' : 'Nothing to show here.'}</p>
  {/each}
</div>

<SelectionPopover />
