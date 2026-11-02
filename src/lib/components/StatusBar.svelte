<script lang="ts">
  import { lspName } from '../lsp';
  import { app } from '../state.svelte';
  import ActivityPanel from './ActivityPanel.svelte';
  import AiSettings from './AiSettings.svelte';
  import ProgressBar from './ProgressBar.svelte';

  let showAi = $state(false);
  const b = $derived(app.snap?.backend);

  const p = $derived(app.overall);
  const lspText = $derived.by(() => {
    const l = app.lsp;
    if (l.state === 'indexing') return `${lspName()}: ${l.detail || 'indexing'}`;
    if (l.state === 'failed') return `${lspName()} failed: ${l.detail}`;
    if (l.state === 'off') return 'LSP off';
    if (l.state === 'missing') return `${lspName()}: not installed`;
    if (l.state === 'installing') return `${lspName()}: installing…`;
    return `${lspName()}: ${l.state}`;
  });
</script>

<footer class="statusbar">
  <ProgressBar {p} thin />
  <span>{p.accepted}/{p.total} lines accepted · {p.deferred} later · {p.pending} pending</span>
  <span class="spacer"></span>
  {#if app.snap?.planning}<span><span class="spinner"></span> planning</span>{/if}
  <ActivityPanel />
  {#if app.lsp.state === 'missing' || app.lsp.state === 'failed'}
    <button class="ghost small lsp {app.lsp.state}" title="{lspText} — click for options" onclick={() => (app.lspPrompt = 'open')}>{lspText}</button>
  {:else}
    <span class="lsp {app.lsp.state}" title={lspText}>{lspText}</span>
  {/if}
  <button class="ghost small" onclick={() => (showAi = true)} title="Change AI, model and reasoning; see project instructions">
    AI: {b?.engine.kind}{b?.model ? ` · ${b.model}` : ''}{b?.effort ? ` · ${b.effort}` : ''} · {app.snap?.instructions.length ?? 0} instr. ▾
  </button>
{#if showAi}<AiSettings close={() => (showAi = false)} />{/if}
</footer>
