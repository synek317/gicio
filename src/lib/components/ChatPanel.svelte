<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { run } from '../actions';
  import { api } from '../api';
  import { renderMarkdown } from '../markdown';
  import { app } from '../state.svelte';
  import type { Message } from '../types';
  import EditHint from './EditHint.svelte';
  import JobLog from './JobLog.svelte';

  let body = $state('');
  let allowEdits = $state(false);
  let list: HTMLElement | undefined = $state();

  const s = $derived(app.snap!);
  const chatJob = $derived(app.jobs.findLast((j) => j.kind === 'chat' && j.status === 'running'));
  const sumJob = $derived(app.jobs.findLast((j) => j.kind === 'summary' && j.status === 'running'));

  $effect(() => {
    void s.chat.length;
    void s.chatBusy;
    if (list) list.scrollTop = list.scrollHeight;
  });

  function who(m: Message) {
    return m.author.kind === 'human' ? 'You' : m.author.kind === 'ai' ? m.author.backend : 'gicio';
  }

  async function send() {
    if (!body.trim() || s.chatBusy) return;
    const text = body;
    body = '';
    await run(api.chatSend(text, allowEdits));
  }

  function key(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      send();
    }
  }

  function links(e: MouseEvent) {
    const a = (e.target as HTMLElement).closest('a');
    if (!a) return;
    e.preventDefault();
    if (/^https?:/.test(a.href)) openUrl(a.href);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="chat" onclick={links}>
  <section class="card summary-card">
    <header class="row">
      <h3>Summary of the change</h3>
      <span class="spacer"></span>
      {#if s.summaryBusy}
        <span class="spinner"></span>
        {#if sumJob}<button class="small danger" onclick={() => api.cancelJob(sumJob.id)}>Stop</button>{/if}
      {:else}
        <button class="small" class:primary={!s.summary} onclick={() => api.summarize().catch((e) => app.toast(String(e), 'error'))}>{s.summary ? 'Regenerate' : 'Generate summary'}</button>
      {/if}
    </header>
    {#if s.summaryBusy && sumJob}<JobLog job={sumJob} lines={4} />{/if}
    {#if s.summary}
      <div class="md">{@html renderMarkdown(s.summary.body)}</div>
    {:else if !s.summaryBusy}
      <p class="muted small">What the change does, how, its risks and where to look closely, written by the AI from the whole diff.</p>
    {/if}
  </section>

  <div class="chat-list" bind:this={list}>
    {#each s.chat as m}
      <div class="msg {m.author.kind} {m.kind}"><div class="who">{who(m)}</div><div class="md">{@html renderMarkdown(m.body)}</div></div>
    {:else}
      <p class="muted small hint">Ask anything about this change as a whole: design, alternatives, what to check, how parts fit together. The AI sees the list of files, the review steps and your progress, and can read the code.</p>
    {/each}
    {#if s.chatBusy}
      <div class="msg ai busy">
        <div class="row"><span class="spinner"></span> thinking… <span class="spacer"></span>{#if chatJob}<button class="small danger" onclick={() => api.cancelJob(chatJob.id)}>Stop</button>{/if}</div>
        {#if chatJob}<JobLog job={chatJob} lines={4} />{/if}
      </div>
    {/if}
  </div>

  <div class="chat-compose">
    <textarea bind:value={body} onkeydown={key} rows="3" placeholder="Ask about the whole change…"></textarea>
    <div class="row">
      <EditHint bind:checked={allowEdits} />
      <span class="spacer"></span>
      {#if s.chat.length}<button class="ghost small" onclick={() => run(api.clearChat())}>Clear</button>{/if}
      <button class="primary" disabled={s.chatBusy} onclick={send}>Send <kbd>⌘↵</kbd></button>
    </div>
  </div>
</div>
