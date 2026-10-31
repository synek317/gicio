<script lang="ts">
  import { onMount } from 'svelte';
  import { currentChunkId } from '../actions';
  import { api } from '../api';
  import { settings } from '../app-settings.svelte';
  import { app } from '../state.svelte';

  const r = $derived(app.rename!);
  let name = $state('');
  let original = $state('');
  let error = $state('');
  let busy = $state(false);
  let keep = $state(settings.value.prefs.renameAccepted !== 'review');
  let dontAsk = $state(false);
  let input: HTMLInputElement | undefined = $state();
  const ask = settings.value.prefs.renameAccepted === 'ask';

  onMount(async () => {
    try {
      const current = await api.lspPrepareRename(r.path, r.line - 1, r.character);
      if (!current) error = 'There is nothing to rename here.';
      else {
        name = original = current;
        requestAnimationFrame(() => input?.select());
      }
    } catch (e) {
      error = String(e);
    }
  });

  const close = () => (app.rename = null);

  async function rename() {
    if (!name.trim() || name === original || busy) return;
    if (app.editorDirty) return (error = 'Save or discard your edits in the open file first.');
    busy = true;
    error = '';
    try {
      if (ask && dontAsk) await settings.save({ ...settings.value, prefs: { ...settings.value.prefs, renameAccepted: keep ? 'keep' : 'review' } });
      const res = await api.lspRename(r.path, r.line - 1, r.character, name.trim(), keep, currentChunkId());
      app.fileRev++;
      app.toast(`Renamed ${original} → ${name.trim()}: ${res.edits} change(s) in ${res.files} file(s).`);
      close();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && close()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="modal-bg" onclick={close}>
  <div class="modal rename-dialog" onclick={(e) => e.stopPropagation()}>
    <h2>Rename{#if original} <code>{original}</code>{/if}</h2>
    <p class="muted small">Everywhere in the project (rust-analyzer). Files are changed on disk; lines the rename adds to the change count as your own edit.</p>
    <input bind:this={input} bind:value={name} onkeydown={(e) => e.key === 'Enter' && rename()} spellcheck="false" disabled={!original || busy} />
    {#if ask}
      <fieldset>
        <legend>Lines you had already accepted</legend>
        <label class="choice"><input type="radio" bind:group={keep} value={true} /> <span>stay accepted</span></label>
        <label class="choice"><input type="radio" bind:group={keep} value={false} /> <span>need another look</span></label>
        <label class="toggle"><input type="checkbox" bind:checked={dontAsk} /> Don't ask me again</label>
      </fieldset>
    {:else}
      <p class="muted small">Lines you had already accepted {keep ? 'stay accepted' : 'need another look'} (Settings → Preferences).</p>
    {/if}
    {#if error}<div class="error">{error}</div>{/if}
    <div class="row"><span class="spacer"></span><button onclick={close}>Cancel</button><button class="primary" disabled={!original || busy || !name.trim() || name === original} onclick={rename}>{busy ? 'Renaming…' : 'Rename'} <kbd>↵</kbd></button></div>
  </div>
</div>
