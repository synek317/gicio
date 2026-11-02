<script lang="ts">
  import { untrack } from 'svelte';
  import { startThread } from '../actions';
  import { app } from '../state.svelte';
  import Composer from './Composer.svelte';

  const initial = app.composerDraft;
  app.composerDraft = '';
  const a = $derived(app.composer?.anchor);
  // Picking other lines while the composer is open moves it there; the cursor follows.
  let focus = $state(0);
  $effect(() => {
    void app.composer?.anchor;
    untrack(() => focus++);
  });
</script>

<div class="card composer">
  <div class="row">
    <span class="anchor muted">{#if a}On <code>{a.path}:{a.side === 'old' ? '−' : '+'}{a.start}{a.end > a.start ? `–${a.end}` : ''}</code>{:else}About the whole step{/if}</span>
    <span class="spacer"></span>
    <button class="ghost small" onclick={() => (app.composer = null)} title="Cancel (Esc)">✕</button>
  </div>
  <Composer send={startThread} {initial} {focus} autofocus placeholder="Question, doubt or change request…" />
</div>
