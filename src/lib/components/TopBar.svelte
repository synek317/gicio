<script lang="ts">
  import { back, forward } from '../actions';
  import { app } from '../state.svelte';
  import { settings } from '../app-settings.svelte';
  import CommitDialog from './CommitDialog.svelte';
  import ReviewMenu from './ReviewMenu.svelte';
  import ToolsMenu from './ToolsMenu.svelte';

  const s = $derived(app.snap!);
  const repoName = $derived(s.repo.split(/[\\/]/).pop());
  let showCommit = $state(false);
</script>

<header class="topbar">
  <div class="seg">
    <button disabled={!app.history.length} onclick={back} title="Back (⌘[ or ⌘−)">←</button>
    <button disabled={!app.future.length} onclick={forward} title="Forward (⌘] or ⌘=)">→</button>
  </div>
  <strong class="brand">gicio</strong>
  <span class="repo" title={s.repo}>{repoName}</span>
  <span class="muted">vs</span>
  <code title={s.baseCommit}>{s.baseRef === 'HEAD' ? 'HEAD (uncommitted)' : s.baseRef}</code>
  <span class="spacer"></span>
  <div class="seg">
    <button class:active={app.layout === 'split'} onclick={() => (app.layout = 'split')}>Split</button>
    <button class:active={app.layout === 'unified'} onclick={() => (app.layout = 'unified')}>Unified</button>
  </div>
  {#if s.stageMode === 'index'}
    <button onclick={() => (showCommit = true)} title="Accepted lines are staged in git; commit them">Commit…</button>
  {/if}
  <ToolsMenu />
  <ReviewMenu />
  <button class="ghost" onclick={() => (settings.open = true)} title="Settings">⚙</button>
  <button class="ghost" onclick={() => (app.showHelp = true)} title="Keyboard shortcuts">?</button>
</header>

{#if showCommit}<CommitDialog close={() => (showCommit = false)} />{/if}
