<script lang="ts">
  import { app } from '../state.svelte';
  import AiSettings from './AiSettings.svelte';
  import ProgressBar from './ProgressBar.svelte';

  let showAi = $state(false);
  const b = $derived(app.snap?.backend);

  const p = $derived(app.overall);
  const busy = $derived(app.snap?.threads.filter((t) => t.busy).length ?? 0);
  const lspText = $derived.by(() => {
    const l = app.lsp;
    if (l.state === 'indexing') return `rust-analyzer: ${l.detail || 'indexing'}`;
    if (l.state === 'failed') return `rust-analyzer failed: ${l.detail}`;
    if (l.state === 'off') return 'LSP off';
    return `rust-analyzer: ${l.state}`;
  });
</script>

<footer class="statusbar">
  <ProgressBar {p} thin />
  <span>{p.accepted}/{p.total} lines accepted · {p.deferred} later · {p.pending} pending</span>
  <span class="spacer"></span>
  {#if app.snap?.planning}<span><span class="spinner"></span> planning</span>{/if}
  {#if busy}<span><span class="spinner"></span> {busy} AI reply in progress</span>{/if}
  <span class="lsp {app.lsp.state}" title={lspText}>{lspText}</span>
  <button class="ghost small" onclick={() => (showAi = true)} title="Change AI, model and reasoning; see project instructions">
    AI: {b?.engine.kind}{b?.model ? ` · ${b.model}` : ''}{b?.effort ? ` · ${b.effort}` : ''} · {app.snap?.instructions.length ?? 0} instr. ▾
  </button>
{#if showAi}<AiSettings close={() => (showAi = false)} />{/if}
</footer>
