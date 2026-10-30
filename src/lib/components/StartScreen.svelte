<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { go, plan } from '../actions';
  import { api, inTauri } from '../api';
  import { loadSettings, saveSettings } from '../settings';
  import { app } from '../state.svelte';
  import { toBackend, type BackendForm } from '../backend-form';
  import BackendPicker from './BackendPicker.svelte';
  import { settings } from '../app-settings.svelte';

  const s = $state(loadSettings());
  let form = $state<BackendForm>({ ...settings.value.defaultBackend });
  s.order = settings.value.prefs.order;
  let refs = $state<string[]>([]);
  let opening = $state(false);
  let error = $state('');

  $effect(() => {
    const r = s.repo;
    if (!r) return;
    api.listRefs(r).then((x) => (refs = x)).catch(() => (refs = []));
  });

  async function browse() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === 'string') s.repo = dir;
  }

  async function start() {
    error = '';
    opening = true;
    Object.assign(s, form);
    const backend = toBackend(form);
    try {
      s.recent = [s.repo, ...s.recent.filter((r) => r !== s.repo)].slice(0, 8);
      saveSettings($state.snapshot(s));
      const stageMode = s.scope === 'uncommitted' ? s.stage : 'memory';
      const snap = await api.openSession(s.repo, s.scope === 'uncommitted' ? 'HEAD' : s.base, backend, stageMode);
      app.layout = settings.value.prefs.layout;
      app.applySnapshot(snap);
      app.history = [];
      app.future = [];
      if (snap.chunks[0]) go({ kind: 'chunk', id: snap.chunks[0].id }, false);
      if (s.order === 'ai' && !snap.plannedByAi && snap.files.length) plan(false);
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
        <input bind:value={s.repo} placeholder="/path/to/repo" spellcheck="false" />
        {#if inTauri}<button onclick={browse}>Browse…</button>{/if}
      </div>
    </label>
    {#if s.recent.length}
      <div class="chips">{#each s.recent as r}<button class="chip" onclick={() => (s.repo = r)} title={r}>{r.split('/').pop()}</button>{/each}</div>
    {/if}

    <fieldset>
      <legend>What to review</legend>
      <label class="choice"><input type="radio" bind:group={s.scope} value="uncommitted" />
        <span><strong>Uncommitted changes</strong><br /><span class="muted small">Staged, unstaged and new files, compared to HEAD.</span></span>
      </label>
      <label class="choice"><input type="radio" bind:group={s.scope} value="since" />
        <span><strong>Everything since a branch or commit</strong><br /><span class="muted small">Commits on top of it plus uncommitted changes, e.g. an AI's branch against <code>main</code>.</span></span>
      </label>
      {#if s.scope === 'since'}
        <div class="indent">
          <input bind:value={s.base} list="refs" spellcheck="false" placeholder="main" />
          <datalist id="refs">{#each refs as r}<option value={r}></option>{/each}</datalist>
          <div class="chips">{#each ['main', 'master', 'origin/main', 'HEAD~1'] as r}<button class="chip" onclick={() => (s.base = r)}>{r}</button>{/each}</div>
        </div>
      {/if}
    </fieldset>

    <fieldset>
      <legend>Accepted changes</legend>
      <label class="choice"><input type="radio" bind:group={s.stage} value="memory" />
        <span><strong>Remember them in gicio only</strong><br /><span class="muted small">Your git index is not touched.</span></span>
      </label>
      <label class="choice" class:disabled={s.scope !== 'uncommitted'}><input type="radio" bind:group={s.stage} value="index" disabled={s.scope !== 'uncommitted'} />
        <span><strong>Stage them in git</strong><br /><span class="muted small">
          {#if s.scope !== 'uncommitted'}Only for uncommitted changes.{:else}Accepted lines are staged (also parts of files), everything else is not, so you can commit exactly what you reviewed.
          <span class="warn-text">Everything currently staged is unstaged first.</span>{/if}
        </span></span>
      </label>
    </fieldset>

    <fieldset>
      <legend>Order of review</legend>
      <label class="choice"><input type="radio" bind:group={s.order} value="ai" />
        <span><strong>Logical steps planned by AI</strong><br /><span class="muted small">Groups related changes (also parts of files) and orders them so each step builds on the previous one. Takes a minute; you can start file by file meanwhile.</span></span>
      </label>
      <label class="choice"><input type="radio" bind:group={s.order} value="files" />
        <span><strong>File by file</strong><br /><span class="muted small">No planning. You can still ask the AI to plan later from the top bar.</span></span>
      </label>
    </fieldset>

    <BackendPicker bind:form />

    {#if error}<div class="error">{error}</div>{/if}
    <button class="primary big" disabled={!s.repo || (s.scope === 'since' && !s.base) || opening} onclick={start}>{opening ? 'Opening…' : 'Start review'}</button>
  </div>
</div>
