<script lang="ts">
  import { api, inTauri } from '../api';
  import { app } from '../state.svelte';
  import type { FinishAction } from '../types';

  /** Shown when everything is reviewed: what to do with the accepted changes, and whether to close gicio. */
  const ACTIONS: { id: FinishAction; label: string; hint: string }[] = [
    { id: 'nothing', label: 'Do nothing', hint: 'leave git as it is' },
    { id: 'stage', label: 'Stage', hint: 'stage the accepted changes' },
    { id: 'commit', label: 'Commit', hint: 'stage the accepted changes and commit them' },
    { id: 'amend', label: 'Amend last commit', hint: 'stage the accepted changes and add them to the last commit' },
  ];

  let action = $state<FinishAction>(app.snap?.stageMode === 'index' ? 'commit' : 'nothing');
  let message = $state('');
  let amendLoaded = false;
  let busy = $state<'' | 'suggest' | 'apply'>('');
  const needsMessage = $derived(action === 'commit' || action === 'amend');

  $effect(() => {
    if (action !== 'amend' || amendLoaded) return;
    amendLoaded = true;
    if (!message.trim()) api.lastCommitMessage().then((m) => (message ||= m), (e) => app.toast(String(e), 'error'));
  });

  function close() {
    if (!busy) app.showFinish = false;
  }

  async function suggest() {
    busy = 'suggest';
    try {
      await api.stageAccepted();
      message = await api.suggestCommitMessage();
    } catch (e) {
      app.toast(String(e), 'error');
    }
    busy = '';
  }

  async function finish(quit: boolean) {
    busy = 'apply';
    try {
      const [done, snap] = await api.finishReview(action, needsMessage ? message : null);
      app.applySnapshot(snap);
      app.toast(done);
      busy = '';
      app.showFinish = false;
      if (!quit) return;
      if (inTauri) await api.quitApp();
      else {
        app.snap = null;
        app.view = null;
      }
    } catch (e) {
      app.toast(String(e), 'error');
      busy = '';
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && close()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="modal-bg" onclick={close}>
  <div class="modal finish-dialog" onclick={(e) => e.stopPropagation()}>
    <h2>{app.finished ? 'Review finished' : 'Finish review'}</h2>
    {#if app.finished}
      <p class="muted">Every change is accepted. What should happen to it?</p>
    {:else}
      <p class="warn-text">Not everything is accepted yet: only the accepted lines are staged or committed.</p>
    {/if}
    <fieldset>
      {#each ACTIONS as a (a.id)}
        <label class="choice">
          <input type="radio" name="finish-action" value={a.id} bind:group={action} disabled={!!busy} />
          <span>{a.label} <span class="muted small">{a.hint}</span></span>
        </label>
      {/each}
    </fieldset>
    {#if needsMessage}
      <textarea rows="7" bind:value={message} placeholder="Commit message" disabled={!!busy}></textarea>
      <div class="row">
        <button disabled={!!busy} onclick={suggest}>{busy === 'suggest' ? 'Writing…' : 'Suggest message with AI'}</button>
      </div>
    {/if}
    <footer class="row">
      <button disabled={!!busy} onclick={close}>Keep reviewing</button>
      <span class="spacer"></span>
      <button disabled={!!busy || action === 'nothing' || (needsMessage && !message.trim())} onclick={() => finish(false)}>Apply</button>
      <button class="primary" disabled={!!busy || (needsMessage && !message.trim())} onclick={() => finish(true)}>
        {busy === 'apply' ? 'Working…' : action === 'nothing' ? 'Close gicio' : 'Apply and close gicio'}
      </button>
    </footer>
  </div>
</div>
