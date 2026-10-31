<script lang="ts">
  import { untrack } from 'svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { revealThread, run } from '../actions';
  import { api } from '../api';
  import { renderMarkdown } from '../markdown';
  import { app } from '../state.svelte';
  import type { Message, SendMode, Thread } from '../types';
  import EditHint from './EditHint.svelte';
  import JobLog from './JobLog.svelte';
  import SendButtons from './SendButtons.svelte';

  let { thread }: { thread: Thread } = $props();

  let reply = $state('');
  let allowEdits = $state(true);
  let expanded = $state(false);
  $effect(() => {
    allowEdits = thread.allowEdits;
  });

  const a = $derived(thread.anchor);
  const active = $derived(app.activeThread === thread.id);

  // Resolved discussions fold away; reopening unfolds them. The user can toggle any time.
  let collapsed = $state(untrack(() => thread.status === 'resolved'));
  let lastStatus = untrack(() => thread.status);
  $effect(() => {
    if (thread.status !== lastStatus) {
      collapsed = thread.status === 'resolved';
      lastStatus = thread.status;
    }
  });

  function who(m: Message) {
    if (m.author.kind === 'human') return 'You';
    if (m.author.kind === 'ai') return m.author.backend;
    return 'gicio';
  }

  async function send(mode: SendMode) {
    if (!reply.trim()) return;
    const text = reply;
    reply = '';
    await run(api.sendMessage(thread.id, text, mode, allowEdits));
  }

  const job = $derived(app.jobForThread(thread.id));

  function key(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      send('ask');
    }
  }

  function links(e: MouseEvent) {
    const link = (e.target as HTMLElement).closest('a');
    if (!link) return;
    e.preventDefault();
    if (/^https?:/.test(link.href)) openUrl(link.href);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="card thread {thread.status}" class:active onclick={() => (app.activeThread = thread.id)}>
  <header class="thread-head">
    <button class="ghost chevron" onclick={(e) => { e.stopPropagation(); collapsed = !collapsed; }} title={collapsed ? 'Expand' : 'Collapse'}>{collapsed ? '▸' : '▾'}</button>
    <button class="link anchor" class:gone={a?.outdated} onclick={() => revealThread(thread)} title={a?.outdated ? 'This code has changed or been removed' : 'Show in the diff'}>
      {#if a}{a.path}:{a.side === 'old' ? '−' : '+'}{a.start}{a.end > a.start ? `–${a.end}` : ''}{:else if thread.chunk}{app.chunk(thread.chunk)?.title ?? 'step'}{:else}Whole review{/if}
    </button>
    {#if a?.outdated}<span class="badge outdated" title="The code this discussion was about has changed or been removed">outdated</span>{/if}
    <span class="badge st-{thread.status}">{thread.status}</span>
    {#if thread.todo}<span class="badge ai">AI to-do</span>{/if}
    <span class="thread-actions">
      {#if thread.status !== 'resolved'}<button class="small resolve" onclick={(e) => { e.stopPropagation(); run(api.setThreadStatus(thread.id, 'resolved')); }} title="Mark this discussion as done">✓ Resolve</button>{/if}
      {#if thread.status === 'open'}<button class="ghost small" onclick={(e) => { e.stopPropagation(); run(api.setThreadStatus(thread.id, 'deferred')); }} title="Come back to it later">Later</button>{/if}
      {#if thread.status !== 'open'}<button class="ghost small" onclick={(e) => { e.stopPropagation(); run(api.setThreadStatus(thread.id, 'open')); }}>Reopen</button>{/if}
    </span>
  </header>
  {#if a?.excerpt && !collapsed}
    <button class="excerpt" class:expanded onclick={() => (expanded = !expanded)}><pre>{a.excerpt}</pre></button>
  {/if}
  {#if !collapsed}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="messages" onclick={links}>
      {#each thread.messages as m}
        <div class="msg {m.author.kind} {m.kind}" class:pending={m.pending}>
          <div class="who">{who(m)}{#if m.pending} <span class="badge">pending review</span>{/if}</div>
          <div class="md">{@html renderMarkdown(m.body)}</div>
        </div>
      {/each}
      {#if thread.busy}
        <div class="msg ai busy">
          <div class="row">
            <span class="spinner"></span>
            {job?.status === 'queued' ? 'Waiting for earlier AI to-dos…' : `${app.snap?.backend.engine.kind} is working${thread.allowEdits || thread.todo ? ' (may edit code)' : ''}…`}
            <span class="spacer"></span>
            {#if job}<button class="small danger" onclick={(e) => { e.stopPropagation(); api.cancelJob(job.id); }}>Stop</button>{/if}
          </div>
          {#if job?.status === 'running'}<JobLog {job} compact />{/if}
        </div>
      {/if}
    </div>
    {#if !thread.busy}
      <textarea class="reply" bind:value={reply} onkeydown={key} rows={reply ? 3 : 1} placeholder="Reply… (⌘↵ asks the AI)"></textarea>
      {#if reply.trim()}
        <div class="send-row"><EditHint bind:checked={allowEdits} /><span class="spacer"></span><SendButtons {send} small /></div>
      {/if}
    {/if}
  {:else}
    <button class="link collapsed-summary" onclick={(e) => { e.stopPropagation(); collapsed = false; }} title="Expand">
      {thread.messages[0]?.body.slice(0, 110) ?? '(empty)'}{#if thread.messages.length > 1} <span class="muted">· {thread.messages.length} messages</span>{/if}
    </button>
  {/if}
</div>
