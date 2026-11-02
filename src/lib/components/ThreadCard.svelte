<script lang="ts">
  import { untrack } from 'svelte';
  import { revealThread, run } from '../actions';
  import { api } from '../api';
  import { app } from '../state.svelte';
  import type { SendMode, Thread } from '../types';
  import Composer from './Composer.svelte';
  import Messages from './Messages.svelte';

  let { thread }: { thread: Thread } = $props();

  let allowEdits = $state(true);
  let expanded = $state(false);
  $effect(() => {
    allowEdits = thread.allowEdits;
  });

  const a = $derived(thread.anchor);
  const active = $derived(app.activeThread === thread.id);
  const job = $derived(app.jobForThread(thread.id));
  const label = { open: 'open', deferred: 'later', resolved: 'resolved' };

  // Resolved conversations fold away; reopening unfolds them. The user can toggle any time.
  let collapsed = $state(untrack(() => thread.status === 'resolved'));
  let lastStatus = untrack(() => thread.status);
  $effect(() => {
    if (thread.status !== lastStatus) {
      collapsed = thread.status === 'resolved';
      lastStatus = thread.status;
    }
  });

  function send(mode: SendMode, text: string, edits: boolean) {
    return run(api.sendMessage(thread.id, text, mode, edits));
  }

  function setStatus(e: MouseEvent, status: Thread['status']) {
    e.stopPropagation();
    run(api.setThreadStatus(thread.id, status));
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="card thread {thread.status}" class:active onclick={() => (app.activeThread = thread.id)}>
  <header class="thread-head">
    <button class="ghost chevron" onclick={(e) => { e.stopPropagation(); collapsed = !collapsed; }} title={collapsed ? 'Expand' : 'Collapse'}>{collapsed ? '▸' : '▾'}</button>
    <button class="link anchor" class:gone={a?.outdated} onclick={() => revealThread(thread)} title={a?.outdated ? 'This code has changed or been removed' : 'Show in the diff'}>
      {#if a}{a.path}:{a.side === 'old' ? '−' : '+'}{a.start}{a.end > a.start ? `–${a.end}` : ''}{:else if thread.chunk}{app.chunk(thread.chunk)?.title ?? 'step'}{:else}Whole change{/if}
    </button>
    {#if a?.outdated}<span class="badge outdated" title="The code this conversation was about has changed or been removed">outdated</span>{/if}
    <span class="badge st-{thread.status}">{label[thread.status]}</span>
    <span class="thread-actions">
      {#if thread.status !== 'resolved'}<button class="small resolve" onclick={(e) => setStatus(e, 'resolved')} title="Mark as done">✓ Resolve</button>{/if}
      {#if thread.status === 'open'}<button class="ghost small" onclick={(e) => setStatus(e, 'deferred')} title="Leave it for the “Resolve issues left for later” step">Later</button>{/if}
      {#if thread.status !== 'open'}<button class="ghost small" onclick={(e) => setStatus(e, 'open')}>Reopen</button>{/if}
    </span>
  </header>
  {#if collapsed}
    <button class="link collapsed-summary" onclick={(e) => { e.stopPropagation(); collapsed = false; }} title="Expand">
      {thread.messages[0]?.body.slice(0, 110) ?? '(empty)'}{#if thread.messages.length > 1} <span class="muted">· {thread.messages.length} messages</span>{/if}
    </button>
  {:else}
    {#if a?.excerpt}
      <button class="excerpt" class:expanded onclick={() => (expanded = !expanded)}><pre>{a.excerpt}</pre></button>
    {/if}
    <Messages messages={thread.messages} busy={thread.busy} {job} busyText="{app.snap?.backend.engine.kind} is working{thread.allowEdits ? ' (may edit code)' : ''}…" />
    {#if !thread.busy}
      <Composer {send} bind:allowEdits reply oneLine placeholder="Reply… (⌘↵ asks the AI)" />
    {/if}
  {/if}
</div>
