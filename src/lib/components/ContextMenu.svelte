<script lang="ts">
  import { editLine, markKeys, openComposer, selectionAnchor, selectLine } from '../actions';
  import { itemsAround } from '../item-keys';
  import { itemLabel } from '../items';
  import { previewOff, previewOn } from '../preview';
  import { texts } from '../texts.svelte';
  import { keyOf } from '../keys';
  import { findReferences, gotoDefinition, lspHandles, lspName, lspUsable } from '../lsp';
  import { app } from '../state.svelte';
  import { reviewWithAi } from '../review-actions';

  const m = $derived(app.menu!);
  const lsp = $derived(m.side === 'new' && lspUsable(m.path));
  const inDiff = $derived(app.orderedKeys.includes(keyOf(m.path, m.side, m.line)));
  /** Whole items around the clicked line that still have unaccepted changes. */
  const items = $derived.by(() => {
    const t = texts.entries[m.path]?.texts;
    const file = app.files.get(m.path);
    if (!t || t.binary || !file) return [];
    const found = m.side === 'new' ? itemsAround(file, t, [m.line], []) : itemsAround(file, t, [], [m.line]);
    return found.map((it) => ({ ...it, open: it.keys.filter((k) => app.marks.get(k) !== 'accepted') })).filter((it) => it.open.length).slice(0, 5);
  });

  /** Runs an action, then closes the menu. The action reads the clicked position (`m`) first:
   *  closing earlier would clear it. */
  function act(f: () => void) {
    try {
      f();
    } finally {
      app.menu = null;
    }
  }

  const selected = $derived(app.selSet.has(keyOf(m.path, m.side, m.line)) && app.selection.length > 1);

  /** The selected lines when the click is inside the selection, else just this line. */
  function reviewHere() {
    const a = selected ? selectionAnchor() : null;
    reviewWithAi(a ? { kind: 'lines', ...a } : { kind: 'lines', path: m.path, side: m.side, start: m.line, end: m.line });
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
    <button disabled={!lsp} onclick={() => act(() => (app.rename = { path: m.path, line: m.line, character: m.character }))}>Rename symbol… <kbd>F2</kbd></button>
    {#if items.length}
      <hr />
      {#each items as it}
        <button onmouseenter={() => previewOn(it.open)} onmouseleave={previewOff} onclick={() => act(() => markKeys(it.open, 'accepted', false))}>✓ Accept {it.side === 'old' ? 'removed ' : ''}{itemLabel(it.item)} <span class="muted">{it.open.length} lines</span></button>
      {/each}
    {/if}
    <hr />
    <button disabled={m.side !== 'new'} onclick={() => act(() => editLine(m.path, m.line))}>Edit here <kbd>e</kbd></button>
    {#if app.view?.kind === 'chunk' || app.view?.kind === 'file' || app.view?.kind === 'later'}
      <button onclick={() => act(ask)}>New conversation… <kbd>c</kbd></button>
      <button onclick={() => act(reviewHere)}>AI review {selected ? 'the selection' : 'this line'}</button>
    {/if}
    {#if !lsp && m.side === 'new' && lspHandles(m.path)}
      {#if app.lsp.state === 'missing' || app.lsp.state === 'failed'}
        <button onclick={() => act(() => (app.lspPrompt = 'open'))}>{lspName()} {app.lsp.state === 'missing' ? 'is not installed' : 'failed'}…</button>
      {:else}
        <div class="muted small menu-note">{lspName()}: {app.lsp.state}</div>
      {/if}
    {/if}
  </div>
</div>
