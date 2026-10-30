<script lang="ts">
  import { settings, type AppSettings } from '../app-settings.svelte';
  import BackendPicker from './BackendPicker.svelte';
  import ShortcutEditor from './ShortcutEditor.svelte';

  let draft = $state<AppSettings>($state.snapshot(settings.value) as AppSettings);
  let tab = $state<'ai' | 'prefs' | 'keys'>('ai');
  let saving = $state(false);

  async function save() {
    saving = true;
    await settings.save($state.snapshot(draft) as AppSettings);
    saving = false;
    settings.open = false;
  }

  const close = () => (settings.open = false);
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && !settings.capturing && close()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="modal-bg" onclick={close}>
  <div class="modal settings" onclick={(e) => e.stopPropagation()}>
    <header class="row">
      <h2>Settings</h2>
      <span class="spacer"></span>
      <nav class="seg">
        <button class:active={tab === 'ai'} onclick={() => (tab = 'ai')}>AI</button>
        <button class:active={tab === 'prefs'} onclick={() => (tab = 'prefs')}>Preferences</button>
        <button class:active={tab === 'keys'} onclick={() => (tab = 'keys')}>Shortcuts</button>
      </nav>
    </header>

    <div class="settings-body">
      {#if tab === 'ai'}
        <h3>Defaults for new reviews</h3>
        <BackendPicker bind:form={draft.defaultBackend} />
        <h3>Your instructions for the AI</h3>
        <p class="muted small">Added to every prompt (planning, answers, to-dos, chat, summaries), on top of the project's CLAUDE.md / AGENTS.md etc.</p>
        <textarea rows="7" bind:value={draft.aiInstructions} placeholder="e.g. Prefer newtypes over raw strings. Point out missing tests. Never suggest unwrap() in library code."></textarea>
      {:else if tab === 'prefs'}
        <div class="prefs">
          <label>Default diff layout
            <select bind:value={draft.prefs.layout}><option value="split">Side by side</option><option value="unified">Unified</option></select>
          </label>
          <label>Default order of review
            <select bind:value={draft.prefs.order}><option value="ai">Logical steps planned by AI</option><option value="files">File by file</option></select>
          </label>
          <label class="toggle"><input type="checkbox" bind:checked={draft.prefs.autoAdvance} /> After “Accept rest”, go to the next step automatically</label>
          <label class="toggle"><input type="checkbox" bind:checked={draft.prefs.notesExpanded} /> Show AI notes (?) expanded</label>
          <label class="toggle"><input type="checkbox" bind:checked={draft.prefs.confirmStartOver} /> Ask before “Start over”</label>
          <label>rust-analyzer hover delay (ms)
            <input type="number" min="100" max="3000" step="50" bind:value={draft.prefs.hoverDelayMs} />
          </label>
        </div>
      {:else}
        <ShortcutEditor bind:keys={draft.keybindings} />
      {/if}
    </div>

    <footer class="row"><span class="spacer"></span><button onclick={close}>Cancel</button><button class="primary" disabled={saving} onclick={save}>Save</button></footer>
  </div>
</div>
