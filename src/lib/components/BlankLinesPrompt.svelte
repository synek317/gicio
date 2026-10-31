<script lang="ts">
  import { markKeys } from '../actions';
  import { settings } from '../app-settings.svelte';
  import { app } from '../state.svelte';

  const p = $derived(app.blankPrompt!);
  let dontAsk = $state(false);

  async function answer(accept: boolean) {
    const keys = p.keys;
    app.blankPrompt = null;
    if (dontAsk) await settings.save({ ...settings.value, prefs: { ...settings.value.prefs, blankLines: accept ? 'accept' : 'never' } });
    if (accept) await markKeys(keys, 'accepted', false);
  }
</script>

<svelte:window onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); answer(true); } else if (e.key === 'Escape') answer(false); }} />

<div class="blank-prompt">
  <div>Everything in <code>{p.path}</code> is accepted except {p.keys.length} line{p.keys.length === 1 ? '' : 's'} with nothing to review (blank or only brackets like <code>}</code>). Accept the whole file?</div>
  <div class="row">
    <label class="toggle"><input type="checkbox" bind:checked={dontAsk} /> Don't ask me again</label>
    <span class="spacer"></span>
    <button class="small" onclick={() => answer(false)}>Not now <kbd>esc</kbd></button>
    <button class="small primary" onclick={() => answer(true)}>Accept file <kbd>↵</kbd></button>
  </div>
</div>
