<script lang="ts">
  import { openAiTask, openComposer } from '../actions';
  import { app } from '../state.svelte';
  import ActivityPanel from './ActivityPanel.svelte';
  import ChatPanel from './ChatPanel.svelte';
  import FindingsPanel from './FindingsPanel.svelte';
  import { submitReview } from '../review-actions';
  import NewThread from './NewThread.svelte';
  import ThreadCard from './ThreadCard.svelte';
  import type { Thread } from '../types';

  const openFindings = $derived((app.snap?.findings ?? []).filter((f) => f.status === 'open').length);
  const rank = { open: 0, deferred: 1, resolved: 2 };
  const byStatus = (a: Thread, b: Thread) => rank[a.status] - rank[b.status] || Number(b.id.slice(1)) - Number(a.id.slice(1));

  /** Discussions of what is on screen first; everything else folded below. */
  const split = $derived.by(() => {
    const v = app.view;
    const paths = new Set(app.viewHunks.map((h) => h.path));
    const here = (t: Thread) => t.id === app.activeThread || (!t.chunk && !t.anchor) || (v?.kind === 'chunk' && t.chunk === v.id) || (!!t.anchor && paths.has(t.anchor.path) && v?.kind !== 'chunk');
    const all = app.snap?.threads ?? [];
    return { here: all.filter(here).sort(byStatus), other: all.filter((t) => !here(t)).sort(byStatus) };
  });
</script>

<aside class="threads">
  <header class="threads-head">
    <nav class="tabs right-tabs">
      <button class:active={app.rightTab === 'discussions'} onclick={() => (app.rightTab = 'discussions')}>Discussions</button>
      <button class:active={app.rightTab === 'chat'} onclick={() => (app.rightTab = 'chat')}>Chat{#if app.snap?.chatBusy} <span class="spinner"></span>{/if}</button>
      <button class:active={app.rightTab === 'findings'} onclick={() => (app.rightTab = 'findings')}>Findings{#if openFindings} <span class="count">{openFindings}</span>{/if}</button>
    </nav>
    <span class="spacer"></span>
    {#if app.rightTab === 'discussions'}
      <button onclick={openAiTask} title="Give the AI a task for the whole review, not tied to particular lines (g)">AI task</button>
      <button onclick={() => openComposer()} title="New comment or question about the selected lines (c)">New</button>
    {/if}
  </header>
  {#if app.pendingReview && app.rightTab === 'discussions'}
    <div class="submit-bar row">
      <span>{app.pendingReview} comment(s) in your pending review</span>
      <span class="spacer"></span>
      <button class="primary small" onclick={submitReview} title="Send all pending comments to the AI as one task">Submit review</button>
    </div>
  {/if}
  {#if app.rightTab === 'chat'}
    <ChatPanel />
  {:else if app.rightTab === 'findings'}
    <FindingsPanel />
  {:else}
  <div class="threads-body">
    <ActivityPanel />
    {#if app.composer}<NewThread />{/if}
    {#each split.here as t (t.id)}
      <ThreadCard thread={t} />
    {:else}
      {#if !app.composer}
        <p class="muted hint">Select lines in the gutter and press <kbd>c</kbd> to write a note, ask the AI now, or add an AI to-do that runs in the background. If the AI edits code, the diff and the queue update.</p>
      {/if}
    {/each}
    {#if split.other.length}
      <button class="qhead toggle-head" onclick={() => (app.showAllThreads = !app.showAllThreads)}>
        {app.showAllThreads ? '▾' : '▸'} Other discussions <span class="muted">({split.other.length}, in other steps)</span>
      </button>
      {#if app.showAllThreads}
        {#each split.other as t (t.id)}<ThreadCard thread={t} />{/each}
      {/if}
    {/if}
  </div>
  {/if}
</aside>
