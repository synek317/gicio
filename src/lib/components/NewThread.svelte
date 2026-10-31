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
      send(app.composer?.task ? 'todo' : 'ask');
    }
  }
</script>

<div class="card composer">
  <div class="row">
    <span class="anchor muted">{#if app.composer?.task}<strong>Task for the AI</strong> · whole review{:else if a}On <code>{a.path}:{a.side === 'old' ? '−' : '+'}{a.start}{a.end > a.start ? `–${a.end}` : ''}</code>{:else}About the whole step{/if}</span>
    <span class="spacer"></span>
    <button class="ghost small" onclick={() => (app.composer = null)} title="Cancel (Esc)">✕</button>
  </div>
  <textarea bind:this={input} bind:value={body} onkeydown={key} rows="4" placeholder={app.composer?.task ? 'e.g. Add tests for the error paths in store.rs; rename Foo to Bar everywhere; make all public items documented' : 'Question, doubt or change request…'}></textarea>
  <EditHint bind:checked={allowEdits} />
  <div class="send-row"><SendButtons {send} disabled={sending} task={!!app.composer?.task} /></div>
</div>
