<script lang="ts">
  import { openComposer } from '../actions';
  import { needsWork } from '../intent-actions';
  import { panelPref } from '../panels.svelte';
  import { app } from '../state.svelte';
  import { stepLabel } from '../step-labels';
  import FindingsPanel from './FindingsPanel.svelte';
  import NewThread from './NewThread.svelte';
  import TaskPanel from './TaskPanel.svelte';
  import ThreadCard from './ThreadCard.svelte';
  import WholeChange from './WholeChange.svelte';
  import type { Thread } from '../types';

  const openFindings = $derived((app.snap?.findings ?? []).filter((f) => f.status === 'open').length);
  const unmet = $derived((app.snap?.requirements ?? []).filter(needsWork).length);
  const rank = { open: 0, deferred: 1, resolved: 2 };
  const byStatus = (a: Thread, b: Thread) => rank[a.status] - rank[b.status] || Number(b.id.slice(1)) - Number(a.id.slice(1));
  let showResolved = $state(false);

  /** Conversations about the whole change: not tied to a step or to lines. */
  const isWhole = (t: Thread) => !t.chunk && !t.anchor;

  /** What the "here" history covers: the step on screen, the issues left for later, or the file. */
  const hereLabel = $derived.by(() => {
    const v = app.view;
    if (v?.kind === 'chunk') return `Step ${stepLabel(app.snap?.chunks ?? [], v.id) ?? ''}`;
    if (v?.kind === 'later') return 'Left for later';
    if (v?.kind === 'file') return 'This file';
    if (v?.kind === 'change') return 'This change';
    return 'Here';
  });

  /** The history shown: open and later ones; resolved ones folded below (the one just resolved stays). */
  const list = $derived.by(() => {
    const v = app.view;
    const paths = new Set(app.viewHunks.map((h) => h.path));
    const here = (t: Thread) =>
      v?.kind === 'chunk'
        ? t.chunk === v.id || (!t.chunk && !!t.anchor && paths.has(t.anchor.path))
        : v?.kind === 'later'
          ? t.status === 'deferred'
          : v?.kind === 'change'
            ? app.snap?.changes.find((c) => c.id === v.id)?.thread === t.id
            : !!t.anchor && paths.has(t.anchor.path);
    const mine = (app.snap?.threads ?? []).filter((t) => (app.convScope === 'whole' ? isWhole(t) : !isWhole(t) && here(t)));
    const shown = (t: Thread) => t.status !== 'resolved' || t.id === app.activeThread;
    return { live: mine.filter(shown).sort(byStatus), resolved: mine.filter((t) => !shown(t)).sort(byStatus) };
  });
  const count = (whole: boolean) => (app.snap?.threads ?? []).filter((t) => isWhole(t) === whole && t.status !== 'resolved').length;
</script>

<aside class="threads" class:panel-hidden={!panelPref('right').visible}>
  <header class="threads-head">
    <nav class="tabs right-tabs">
      <button class:active={app.rightTab === 'conversations'} onclick={() => (app.rightTab = 'conversations')}>Conversations{#if app.snap?.chatBusy || app.snap?.threads.some((t) => t.busy)} <span class="spinner"></span>{/if}</button>
      <button class:active={app.rightTab === 'findings'} onclick={() => (app.rightTab = 'findings')}>Findings{#if openFindings} <span class="count">{openFindings}</span>{/if}</button>
      <button class:active={app.rightTab === 'task'} onclick={() => (app.rightTab = 'task')} title="What the change was supposed to do, and whether it does">Task{#if app.snap?.checkingIntent} <span class="spinner"></span>{:else if unmet} <span class="count">{unmet}</span>{/if}</button>
    </nav>
    <span class="spacer"></span>
    {#if app.rightTab === 'conversations'}
      <button onclick={() => openComposer()} title="New conversation about the selected lines or this step (c)">New</button>
    {/if}
  </header>
  {#if app.rightTab === 'findings'}
    <FindingsPanel />
  {:else if app.rightTab === 'task'}
    <TaskPanel />
  {:else}
    <div class="seg scope-switch">
      <button class:active={app.convScope === 'here'} onclick={() => (app.convScope = 'here')} title="Conversations about what is on screen">{hereLabel}</button>
      <button class:active={app.convScope === 'whole'} onclick={() => (app.convScope = 'whole')} title="The conversation about the change as a whole (t)">Whole change{#if count(true)} <span class="muted">· {count(true)}</span>{/if}</button>
    </div>
    <div class="threads-body">
      {#if app.convScope === 'whole'}
        <WholeChange />
      {:else if app.composer}
        <NewThread />
      {/if}
      {#each list.live as t (t.id)}
        <ThreadCard thread={t} />
      {:else}
        {#if app.convScope === 'here' && !app.composer}
          <p class="muted hint">No conversations here yet. Select lines in the gutter and type your question in the box that appears next to them: start a conversation with the AI now, or add it for later.</p>
        {/if}
      {/each}
      {#if list.resolved.length}
        <button class="qhead toggle-head" onclick={() => (showResolved = !showResolved)}>
          {showResolved ? '▾' : '▸'} Resolved <span class="muted">({list.resolved.length})</span>
        </button>
        {#if showResolved}
          {#each list.resolved as t (t.id)}<ThreadCard thread={t} />{/each}
        {/if}
      {/if}
    </div>
  {/if}
</aside>
