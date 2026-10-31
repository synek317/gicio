<script lang="ts">
  import { acceptRestAndNext, markKeys, markView, nextPendingChunk, openComposer } from '../actions';
  import { renderMarkdown } from '../markdown';
  import { hunksProgress } from '../progress';
  import { app } from '../state.svelte';
  import type { Hunk } from '../types';
  import FileBlock from './FileBlock.svelte';
  import ProgressBar from './ProgressBar.svelte';
  import SelectionBar from './SelectionBar.svelte';
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

  function unacceptView() {
    markKeys(app.orderedKeys.filter((k) => app.marks.get(k) === 'accepted'), null, false);
  }
  const title = $derived(chunk?.title ?? (v?.kind === 'later' ? 'Put off for later' : v?.kind === 'file' ? v.path : ''));
</script>

<div class="chunk-view" style="--head-h:{headH}px">
  <header class="chunk-head" bind:offsetHeight={headH}>
    <div class="title-row">
      <h1>{title}</h1>
      {#if chunk?.origin.kind === 'ai'}<span class="badge ai">AI plan</span>{/if}
      {#if chunk?.origin.kind === 'followUp'}<span class="badge follow">follow-up</span>{/if}
    </div>
    {#if chunk?.summary}
      <div class="summary md">{@html renderMarkdown(chunk.summary)}</div>
    {:else if v?.kind === 'later'}
      <p class="summary muted">Every hunk that still has lines you put off. Accept them, or reset them back to pending.</p>
    {/if}
    <div class="head-actions">
      <ProgressBar p={prog} />
      <span class="muted">{prog.accepted} accepted · {prog.deferred} later · {prog.pending} pending</span>
      <span class="spacer"></span>
      <button class="ghost" onclick={() => openComposer(false)}>Ask about this chunk</button>
      {#if prog.pending}
        <button onclick={() => markView('deferred')}>Put rest off</button>
        <button class="primary" onmouseenter={() => previewOn(app.orderedKeys)} onmouseleave={previewOff} onclick={acceptRestAndNext} title="Shift+A">Accept rest & next</button>
      {/if}
      {#if prog.accepted}
        <button class="ghost" onclick={unacceptView} title="Take back every acceptance in this step">↺ Unaccept step</button>
      {/if}
    </div>
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
    <p class="empty muted">Nothing to show here.</p>
  {/each}
</div>

<SelectionBar />
