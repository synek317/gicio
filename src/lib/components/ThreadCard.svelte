<script lang="ts">
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
  <header class="row">
    <button class="link anchor" onclick={() => revealThread(thread)}>
      {#if a}{a.path}:{a.side === 'old' ? '−' : '+'}{a.start}{a.end > a.start ? `–${a.end}` : ''}{:else}{app.chunk(thread.chunk ?? '')?.title ?? 'chunk'}{/if}
    </button>
    <span class="badge st-{thread.status}">{thread.status}</span>
    {#if thread.todo}<span class="badge ai">AI to-do</span>{/if}
    <span class="spacer"></span>
    {#if thread.status !== 'resolved'}<button class="ghost small" onclick={() => run(api.setThreadStatus(thread.id, 'resolved'))}>Resolve</button>{/if}
    {#if thread.status === 'open'}<button class="ghost small" onclick={() => run(api.setThreadStatus(thread.id, 'deferred'))}>Later</button>{/if}
    {#if thread.status !== 'open'}<button class="ghost small" onclick={() => run(api.setThreadStatus(thread.id, 'open'))}>Reopen</button>{/if}
  </header>
  {#if a?.excerpt}
    <button class="excerpt" class:expanded onclick={() => (expanded = !expanded)}><pre>{a.excerpt}</pre></button>
  {/if}
  {#if thread.status !== 'resolved' || active}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="messages" onclick={links}>
      {#each thread.messages as m}
        <div class="msg {m.author.kind} {m.kind}">
          <div class="who">{who(m)}</div>
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
          {#if job?.status === 'running'}<JobLog {job} lines={5} />{/if}
        </div>
      {/if}
    </div>
    {#if !thread.busy}
      <textarea bind:value={reply} onkeydown={key} rows="2" placeholder="Reply…"></textarea>
      <EditHint bind:checked={allowEdits} />
      <div class="row"><span class="spacer"></span><SendButtons {send} small /></div>
    {/if}
  {:else}
    <div class="muted small">{thread.messages[0]?.body.slice(0, 120)}</div>
  {/if}
</div>
