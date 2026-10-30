<script lang="ts">
  import { openComposer } from '../actions';
  import { app } from '../state.svelte';
  import ActivityPanel from './ActivityPanel.svelte';
  import ChatPanel from './ChatPanel.svelte';
  import FindingsPanel from './FindingsPanel.svelte';
  import { submitReview } from '../review-actions';
  import NewThread from './NewThread.svelte';
  import ThreadCard from './ThreadCard.svelte';

  const openFindings = $derived((app.snap?.findings ?? []).filter((f) => f.status === 'open').length);
  const visible = $derived.by(() => {
    const all = app.snap?.threads ?? [];
    const v = app.view;
    const paths = new Set(app.viewHunks.map((h) => h.path));
    const shown = app.showAllThreads
      ? all
      : all.filter((t) => t.id === app.activeThread || (v?.kind === 'chunk' && t.chunk === v.id) || (t.anchor && paths.has(t.anchor.path) && v?.kind !== 'chunk'));
    const rank = { open: 0, deferred: 1, resolved: 2 };
    return [...shown].sort((a, b) => rank[a.status] - rank[b.status] || Number(b.id.slice(1)) - Number(a.id.slice(1)));
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
      <label class="toggle"><input type="checkbox" bind:checked={app.showAllThreads} /> all</label>
      <button onclick={() => openComposer()} title="c">New</button>
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
    {#each visible as t (t.id)}
      <ThreadCard thread={t} />
    {:else}
      {#if !app.composer}
        <p class="muted hint">Select lines in the gutter and press <kbd>c</kbd> to write a note, ask the AI now, or add an AI to-do that runs in the background. If the AI edits code, the diff and the queue update.</p>
      {/if}
    {/each}
  </div>
  {/if}
</aside>
