<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api, inTauri } from '../api';
  import { settings } from '../app-settings.svelte';
  import type { BackendForm } from '../backend-form';
  import { openReview } from '../open-review';
  import { loadRecent, saveRecent } from '../settings';
  import type { StartOptions } from '../start-options';
  import BackendPicker from './BackendPicker.svelte';
  import ReviewOptions from './ReviewOptions.svelte';

  const recent = $state(loadRecent());
  let options = $state<StartOptions>({ ...settings.value.start });
  let form = $state<BackendForm>({ ...settings.value.defaultBackend });
  let remember = $state(false);
  let opening = $state(false);
  let error = $state('');

  // `gicio [PATH] [--since REF] [--start]`: the command line wins over the defaults. With remembered
  // defaults (or --start) the review opens right away.
  onMount(async () => {
    const args = await api.launchArgs().catch(() => null);
    if (!args) return;
    if (args.repo) recent.repo = args.repo;
    if (args.since) {
      options.scope = 'since';
      options.base = args.since;
    }
    if (args.warning) error = args.warning;
    if (args.repo && (args.start || settings.value.skipStartScreen)) start();
  });

  async function browse() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === 'string') recent.repo = dir;
  }

  async function start() {
    error = '';
    opening = true;
    try {
      recent.recent = [recent.repo, ...recent.recent.filter((r) => r !== recent.repo)].slice(0, 8);
      saveRecent($state.snapshot(recent));
      if (remember) {
        await settings.save({ ...settings.value, start: $state.snapshot(options), defaultBackend: $state.snapshot(form), skipStartScreen: true });
      }
      await openReview(recent.repo, options, form);
    } catch (e) {
      error = String(e);
    } finally {
      opening = false;
    }
  }
</script>

<div class="start">
  <div class="start-card">
    <div class="row"><h1>gicio</h1><span class="spacer"></span><button class="ghost" onclick={() => (settings.open = true)} title="Settings">⚙ Settings</button></div>
    <p class="muted">Review a big change piece by piece, with an AI that splits it up, answers questions and fixes things.</p>

    <label>Repository
      <div class="row">
        <input bind:value={recent.repo} placeholder="/path/to/repo" spellcheck="false" />
        {#if inTauri}<button onclick={browse}>Browse…</button>{/if}
      </div>
    </label>
    {#if recent.recent.length}
      <div class="chips">{#each recent.recent as r}<button class="chip" onclick={() => (recent.repo = r)} title={r}>{r.split('/').pop()}</button>{/each}</div>
    {/if}

    <ReviewOptions bind:options repo={recent.repo || null} idPrefix="start" />
    <BackendPicker bind:form />

    <label class="toggle remember">
      <input type="checkbox" bind:checked={remember} />
      Remember these choices and skip this screen next time
      <span class="info" title="`gicio` and `gicio PATH` will open the review right away. Change everything later from the “Start over ▾” menu or in Settings → Preferences.">ⓘ</span>
    </label>

    {#if error}<div class="error">{error}</div>{/if}
    <button class="primary big" disabled={!recent.repo || (options.scope === 'since' && !options.base) || opening} onclick={start}>{opening ? 'Opening…' : 'Start review'}</button>
  </div>
</div>
