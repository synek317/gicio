<script lang="ts">
  import { back, forward, plan, run } from '../actions';
  import { api } from '../api';
  import { app } from '../state.svelte';
  import { settings } from '../app-settings.svelte';
  import CommitDialog from './CommitDialog.svelte';
  import StartOverMenu from './StartOverMenu.svelte';
  import ToolsMenu from './ToolsMenu.svelte';

  const s = $derived(app.snap!);
  const repoName = $derived(s.repo.split(/[\\/]/).pop());
  let showCommit = $state(false);
</script>

<header class="topbar">
  <div class="seg">
    <button disabled={!app.history.length} onclick={back} title="Back (⌘[ or Alt+←)">←</button>
    <button disabled={!app.future.length} onclick={forward} title="Forward (⌘] or Alt+→)">→</button>
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
    <span class="badge stage" title="Accepted lines are staged in git; everything else is unstaged">staging accepted</span>
    <button onclick={() => (showCommit = true)}>Commit…</button>
  {/if}
  <ToolsMenu />
  {#if !s.plannedByAi}
    <button class="primary" disabled={s.planning} onclick={() => plan(false)}>{s.planning ? 'Planning…' : 'Plan with AI'}</button>
  {:else}
    <button disabled={s.planning} onclick={() => plan(true)} title="Ask the AI to regroup steps you haven't started yet">{s.planning ? 'Planning…' : 'Re-plan remaining'}</button>
  {/if}
  <button disabled={!s.canUndo} onclick={() => run(api.undo())} title="Undo last accept/later (z)">Undo</button>
  <button onclick={() => run(api.refresh())} title="Reload the diff from disk (r)">Reload</button>
  <StartOverMenu />
  <button class="ghost" onclick={() => (settings.open = true)} title="Settings">⚙</button>
  <button class="ghost" onclick={() => (app.showHelp = true)}>?</button>
  <button class="ghost" onclick={() => { app.snap = null; app.view = null; }} title="Close this review (open another repository)">✕</button>
</header>

{#if showCommit}<CommitDialog close={() => (showCommit = false)} />{/if}
