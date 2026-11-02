<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { api } from '../api';
  import { renderMarkdown } from '../markdown';
  import type { JobInfo, Message } from '../types';
  import { go } from '../nav';
  import { app } from '../state.svelte';
  import ChangeActions from './ChangeActions.svelte';
  import JobLog from './JobLog.svelte';

  /** One conversation: its messages and, while the AI works on it, what it is doing. */
  let { messages, busy = false, busyText, job }: { messages: Message[]; busy?: boolean; busyText: string; job?: JobInfo } = $props();

  function who(m: Message) {
    return m.author.kind === 'human' ? 'You' : m.author.kind === 'ai' ? m.author.backend : 'gicio';
  }

  function links(e: MouseEvent) {
    const a = (e.target as HTMLElement).closest('a');
    if (!a || a.classList.contains('path-link')) return;
    e.preventDefault();
    if (/^https?:/.test(a.href)) openUrl(a.href);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="messages" onclick={links}>
  {#each messages as m}
    <div class="msg {m.author.kind} {m.kind}">
      <div class="who">{who(m)}</div>
      <div class="md">{@html renderMarkdown(m.body)}</div>
      {#if m.change && app.snap?.changes.find((c) => c.id === m.change)}
        {@const change = app.snap.changes.find((c) => c.id === m.change)!}
        <div class="row change-links">
          <button class="small" onclick={(e) => (e.stopPropagation(), go({ kind: 'change', id: change.id }))} title="Show only what the AI changed here">Show changes</button>
          <span class="spacer"></span>
          <ChangeActions {change} small />
        </div>
      {/if}
    </div>
  {/each}
  {#if busy}
    <div class="msg ai busy">
      <div class="row">
        <span class="spinner"></span>
        {job?.status === 'queued' ? 'Waiting for earlier AI work…' : busyText}
        <span class="spacer"></span>
        {#if job}<button class="small danger" onclick={(e) => { e.stopPropagation(); api.cancelJob(job.id); }}>Stop</button>{/if}
      </div>
      {#if job?.status === 'running'}<JobLog {job} compact />{/if}
    </div>
  {/if}
</div>
