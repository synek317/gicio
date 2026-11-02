<script lang="ts">
  import { run, startConversation } from '../actions';
  import { api } from '../api';
  import { app } from '../state.svelte';
  import type { SendMode } from '../types';
  import Composer from './Composer.svelte';
  import Messages from './Messages.svelte';
  import SummaryBox from './SummaryBox.svelte';

  /** The conversation about the change as a whole, with its summary. */
  // Change requests are common here; the choice is remembered on this machine.
  const EDITS_KEY = 'gicio.chat.allowEdits';
  let allowEdits = $state((() => {
    try {
      return localStorage.getItem(EDITS_KEY) !== 'false';
    } catch {
      return true;
    }
  })());
  $effect(() => {
    try {
      localStorage.setItem(EDITS_KEY, String(allowEdits));
    } catch {
      /* private mode */
    }
  });

  const s = $derived(app.snap!);
  const job = $derived(app.jobs.findLast((j) => j.kind === 'chat' && j.status === 'running'));

  function send(mode: SendMode, text: string, edits: boolean) {
    return mode === 'later' ? startConversation(null, null, 'later', text, edits) : run(api.chatSend(text, edits));
  }
</script>

<div class="card thread whole">
  <SummaryBox />
  {#if !s.chat.length && !s.chatBusy}
    <p class="muted small">Ask anything about the change as a whole: design, alternatives, what to check, how parts fit together. The AI sees the list of files, the review steps and your progress, and can read the code.</p>
  {/if}
  <Messages messages={s.chat} busy={s.chatBusy} {job} busyText="thinking…" />
  <Composer {send} bind:allowEdits reply oneLine focus={app.chatFocus} placeholder="Ask about the whole change…" />
  {#if s.chat.length}<button class="link small clear-chat" onclick={() => run(api.clearChat())} title="Forget this conversation">Clear conversation</button>{/if}
</div>
