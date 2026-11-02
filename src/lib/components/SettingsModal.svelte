<script lang="ts">
  import { DEFAULT_SKIP, settings, type AppSettings } from '../app-settings.svelte';
  import BackendPicker from './BackendPicker.svelte';
  import ReviewOptions from './ReviewOptions.svelte';
  import ShortcutEditor from './ShortcutEditor.svelte';

  let draft = $state<AppSettings>($state.snapshot(settings.value) as AppSettings);
  let tab = $state<'ai' | 'prefs' | 'keys'>('ai');
  let saving = $state(false);

  async function save() {
    saving = true;
    draft.skipPatterns = draft.skipPatterns.map((p) => p.trim()).filter(Boolean);
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
        <h3>Files not worth an AI review</h3>
        <label class="toggle"><input type="checkbox" bind:checked={draft.skipLowValue} /> Leave lock files and generated code out of AI planning, AI reviews and summaries</label>
        {#if draft.skipLowValue}
          <p class="muted small">One pattern per line. Without a <code>/</code> it matches the file name anywhere. These files still get reviewed by you; planning puts them in a last step of their own.</p>
          <textarea rows="5" class="mono" value={draft.skipPatterns.join('\n')} oninput={(e) => (draft.skipPatterns = e.currentTarget.value.split('\n'))}></textarea>
          <button class="small ghost" onclick={() => (draft.skipPatterns = [...DEFAULT_SKIP])}>Reset to defaults</button>
        {/if}
        <h3>Your instructions for the AI</h3>
        <p class="muted small">Added to every prompt (planning, answers, to-dos, chat, summaries), on top of the project's CLAUDE.md / AGENTS.md etc.</p>
        <textarea rows="7" bind:value={draft.aiInstructions} placeholder="e.g. Prefer newtypes over raw strings. Point out missing tests. Never suggest unwrap() in library code."></textarea>
      {:else if tab === 'prefs'}
        <h3>Opening a repository</h3>
        <label class="toggle"><input type="checkbox" bind:checked={draft.skipStartScreen} /> Skip the start screen: <code>gicio</code> and <code>gicio PATH</code> open the review right away with these defaults</label>
        <ReviewOptions bind:options={draft.start} idPrefix="settings" />
        <p class="muted small">The AI used by default is set in the AI tab. During a review, “Start over ▾” changes all of this for the current review.</p>
        <h3>Reviewing</h3>
        <div class="prefs">
          <label>Default diff layout
            <select bind:value={draft.prefs.layout}><option value="split">Side by side</option><option value="unified">Unified</option></select>
          </label>
          <label class="toggle"><input type="checkbox" bind:checked={draft.prefs.autoAdvance} /> After “Accept rest”, go to the next step automatically</label>
          <label>When only blank or bracket-only lines (<code>{'}'}</code>, <code>{'{'}</code>, <code>{'});'}</code>) are left unaccepted in a file
            <select bind:value={draft.prefs.blankLines}>
              <option value="ask">Ask whether to accept the whole file</option>
              <option value="accept">Accept them automatically</option>
              <option value="never">Do nothing</option>
            </select>
          </label>
          <label>After a rename, lines you had already accepted
            <select bind:value={draft.prefs.renameAccepted}>
              <option value="ask">Ask each time</option>
              <option value="keep">Stay accepted</option>
              <option value="review">Need another look</option>
            </select>
          </label>
          <label class="toggle"><input type="checkbox" bind:checked={draft.prefs.notesExpanded} /> Show AI notes (?) expanded</label>
          <label class="toggle"><input type="checkbox" bind:checked={draft.prefs.confirmStartOver} /> Ask before “Start over”</label>
          <label class="toggle"><input type="checkbox" bind:checked={draft.prefs.offerLspInstall} /> Offer to install a missing language server (rust-analyzer, csharp-ls)</label>
          <label>Language server hover delay (ms)
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
