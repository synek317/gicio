<script lang="ts">
  import { api } from '../api';
  import { app } from '../state.svelte';

  let { close }: { close: () => void } = $props();
  let message = $state('');
  let stat = $state('');
  let busy = $state<'' | 'suggest' | 'commit'>('');

  api.stagedStat().then((s) => (stat = s), (e) => (stat = String(e)));

  async function suggest() {
    busy = 'suggest';
    try {
      message = await api.suggestCommitMessage();
    } catch (e) {
      app.toast(String(e), 'error');
    }
    busy = '';
  }

  async function commit() {
    busy = 'commit';
    try {
      const [sha, snap] = await api.commit(message);
      app.applySnapshot(snap);
      app.toast(`Committed ${sha}`);
      close();
    } catch (e) {
      app.toast(String(e), 'error');
    }
    busy = '';
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && close()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="modal-bg" onclick={close}>
  <div class="modal commit-dialog" onclick={(e) => e.stopPropagation()}>
    <h2>Commit accepted changes</h2>
    <pre class="job-log">{stat || 'Nothing staged yet: accept some lines first.'}</pre>
    <textarea rows="8" bind:value={message} placeholder="Commit message"></textarea>
    <div class="row">
      <button disabled={!!busy || !stat.trim()} onclick={suggest}>{busy === 'suggest' ? 'Writing…' : 'Suggest message with AI'}</button>
      <span class="spacer"></span>
      <button onclick={close}>Cancel</button>
      <button class="primary" disabled={!!busy || !message.trim() || !stat.trim()} onclick={commit}>Commit</button>
    </div>
  </div>
</div>
