<script lang="ts">
  import { api } from '../api';
  import { settings } from '../app-settings.svelte';
  import { lspName } from '../lsp';
  import { app } from '../state.svelte';

  const l = $derived(app.lsp);
  const files = $derived(l.extensions.map((e) => `.${e}`).join(', '));
  let dontAsk = $state(false);

  async function close() {
    app.lspPrompt = 'closed';
    if (dontAsk) await settings.save({ ...settings.value, prefs: { ...settings.value.prefs, offerLspInstall: false } });
  }

  async function act(f: () => Promise<unknown>, done: string | null, pending: 'installing' | 'starting') {
    const name = lspName();
    await close();
    // Shown at once; the status poll would take up to two seconds.
    app.lsp = { ...app.lsp, state: pending, install: null };
    try {
      await f();
      if (done) app.toast(`${name} ${done}`);
    } catch (e) {
      app.toast(String(e), 'error');
    }
    app.lsp = await api.lspStatus().catch(() => app.lsp);
  }
</script>

<div class="lsp-prompt">
  {#if l.state === 'missing'}
    <div>
      <strong>{lspName()}</strong> is not installed. Without it, {files} files have no hover, go to definition, references or
      rename; the rest of the review works as usual.
    </div>
    {#if l.install?.available}
      <div>Install it now? gicio runs <code>{l.install.command}</code>.</div>
    {:else if l.install}
      <div>It is installed with <code>{l.install.command}</code>, but that needs {l.install.prerequisite} first. Install it, then check again.</div>
    {/if}
  {:else if l.state === 'failed'}
    <div><strong>{lspName()}</strong> failed: {l.detail}</div>
  {/if}
  <div class="row">
    {#if l.state === 'missing'}<label class="toggle"><input type="checkbox" bind:checked={dontAsk} /> Don't ask me again</label>{/if}
    <span class="spacer"></span>
    <button class="small" onclick={close}>Not now</button>
    {#if l.state === 'missing' && l.install?.available}
      <button class="small primary" onclick={() => act(api.lspInstall, 'installed', 'installing')}>Install</button>
    {:else}
      <button class="small primary" onclick={() => act(api.lspRestart, null, 'starting')}>{l.state === 'failed' ? 'Restart' : 'Check again'}</button>
    {/if}
  </div>
</div>
