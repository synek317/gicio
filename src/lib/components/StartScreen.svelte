<script lang="ts">
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { api, inTauri } from '../api';
  import { defaultSettings, settings } from '../app-settings.svelte';
  import type { BackendForm } from '../backend-form';
  import { openReview } from '../open-review';
  import { loadRecent, saveRecent } from '../settings';
  import { takeStartDraft } from '../start-draft';
  import type { StartOptions } from '../start-options';
  import { stepCountReady } from '../step-count';
  import BackendPicker from './BackendPicker.svelte';
  import IntentField from './IntentField.svelte';
  import ReviewOptions from './ReviewOptions.svelte';

  /** Set when a review was started over: its choices come back, so only what should differ needs changing. */
  const draft = takeStartDraft();
  const recent = $state(loadRecent());
  if (draft) recent.repo = draft.repo;
  let options = $state<StartOptions>({ ...(draft?.options ?? settings.value.start) });
  /** Belongs to this one review, so unlike the options it is never remembered as a default. */
  let intent = $state(draft?.intent ?? '');
  let form = $state<BackendForm>({ ...(draft?.form ?? settings.value.defaultBackend) });
  let remember = $state(false);
  let opening = $state(false);
  let error = $state('');
  /** `--fresh`: the review about to open must not continue the saved one. */
  let fresh = false;

  // `gicio [PATH] [--since REF] [--start] [--fresh]`: the command line wins over the defaults. With
  // remembered defaults (or --start) the review opens right away, unless --fresh ignores them.
  onMount(async () => {
    const args = await api.launchArgs().catch(() => null);
    if (!args) return;
    if (args.fresh) {
      const defaults = defaultSettings();
      fresh = true;
      options = defaults.start;
      form = defaults.defaultBackend;
      recent.repo = '';
    }
    if (args.repo) recent.repo = args.repo;
    if (args.since) {
      options.scope = 'since';
      options.base = args.since;
    }
    if (args.warning) error = args.warning;
    if (args.repo && (args.start || (settings.value.skipStartScreen && !args.fresh))) start();
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
      // Starting over keeps the reviewer's description until now; an emptied field means to drop it.
      if (draft?.intent && !intent.trim()) await api.setIntent(null);
      await openReview(recent.repo, options, form, intent, fresh);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
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
      <div class="chips">{#each recent.recent as r}<button class="chip" onclick={() => (recent.repo = r)} title={r}>{r.split(/[\\/]/).pop()}</button>{/each}</div>
    {/if}

    <ReviewOptions bind:options repo={recent.repo || null} idPrefix="start" />
    <div class="intent-field">
      <label for="start-intent"><span>What should this change do? <span class="muted small">optional</span></span></label>
      <IntentField id="start-intent" bind:value={intent} />
    </div>
    <BackendPicker bind:form />

    <label class="toggle remember">
      <input type="checkbox" bind:checked={remember} />
      Remember these choices and skip this screen next time
      <span class="info" title="`gicio` and `gicio PATH` will open the review right away. Change everything later from the “Start over ▾” menu or in Settings → Preferences.">ⓘ</span>
    </label>

    {#if error}<div class="error">{error}</div>{/if}
    <button class="primary big" disabled={!recent.repo || (options.scope === 'since' && !options.base) || (options.order === 'ai' && !stepCountReady(options.steps)) || opening} onclick={start}>{opening ? 'Opening…' : 'Start review'}</button>
  </div>
</div>
