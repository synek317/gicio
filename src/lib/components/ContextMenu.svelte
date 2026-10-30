<script lang="ts">
  import { editFile, openComposer, selectLine } from '../actions';
  import { keyOf } from '../keys';
  import { findReferences, gotoDefinition, lspUsable } from '../lsp';
  import { app } from '../state.svelte';

  const m = $derived(app.menu!);
  const lsp = $derived(m.side === 'new' && lspUsable(m.path));
  const inDiff = $derived(app.orderedKeys.includes(keyOf(m.path, m.side, m.line)));

  function act(f: () => void) {
    app.menu = null;
    f();
  }

  function ask() {
    if (inDiff && !app.selSet.has(keyOf(m.path, m.side, m.line))) selectLine(keyOf(m.path, m.side, m.line), 'set');
    openComposer(inDiff);
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="menu-bg" onclick={() => (app.menu = null)} oncontextmenu={(e) => { e.preventDefault(); app.menu = null; }}>
  <div class="menu" style="left:{Math.min(m.x, window.innerWidth - 230)}px; top:{Math.min(m.y, window.innerHeight - 180)}px" onclick={(e) => e.stopPropagation()}>
    <button disabled={!lsp} onclick={() => act(() => gotoDefinition(m.path, m.line, m.character))}>Go to definition <kbd>⌘-click</kbd></button>
    <button disabled={!lsp} onclick={() => act(() => findReferences(m.path, m.line, m.character))}>Find references</button>
    <hr />
    <button disabled={m.side !== 'new'} onclick={() => act(() => editFile(m.path, m.line))}>Edit here <kbd>e</kbd></button>
    {#if app.view?.kind === 'chunk' || app.view?.kind === 'file' || app.view?.kind === 'later'}
      <button onclick={() => act(ask)}>Ask AI about this…</button>
    {/if}
    {#if !lsp && m.side === 'new' && m.path.endsWith('.rs')}<div class="muted small menu-note">rust-analyzer: {app.lsp.state}</div>{/if}
  </div>
</div>
