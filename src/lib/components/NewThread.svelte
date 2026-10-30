<script lang="ts">
  import { startThread } from '../actions';
  import { app } from '../state.svelte';
  import type { SendMode } from '../types';
  import EditHint from './EditHint.svelte';
  import SendButtons from './SendButtons.svelte';

  let body = $state(app.composerDraft);
  app.composerDraft = '';
  let allowEdits = $state(true);
  let sending = $state(false);
  let input: HTMLTextAreaElement | undefined = $state();
  const a = $derived(app.composer?.anchor);

  $effect(() => {
    void app.composer?.anchor;
    input?.focus();
  });

  async function send(mode: SendMode) {
    if (!body.trim() || sending) return;
    sending = true;
    const text = body;
    body = '';
    await startThread(text, mode, allowEdits);
    sending = false;
  }

  function key(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      send('ask');
    }
  }
</script>

<div class="card composer">
  <div class="anchor muted">
    {#if a}On <code>{a.path}:{a.side === 'old' ? '−' : '+'}{a.start}{a.end > a.start ? `–${a.end}` : ''}</code>{:else}About the whole step{/if}
  </div>
  <textarea bind:this={input} bind:value={body} onkeydown={key} rows="4" placeholder="Question, doubt or change request…"></textarea>
  <EditHint bind:checked={allowEdits} />
  <div class="row">
    <button class="ghost" onclick={() => (app.composer = null)}>Cancel</button>
    <span class="spacer"></span>
    <SendButtons {send} disabled={sending} />
  </div>
</div>
