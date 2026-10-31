<script lang="ts">
  import { editFile, focusHunk, markKeys } from '../actions';
  import { changedKeys } from '../keys';
  import { hunkProgress } from '../progress';
  import { splitRows, unifiedRows } from '../rows';
  import { app } from '../state.svelte';
  import type { Hunk } from '../types';
  import LineCell from './LineCell.svelte';
  import { enclosingRange, extendHunk, room } from '../expand';
  import { texts } from '../texts.svelte';
  import { itemsAround } from '../item-keys';
  import { itemLabel } from '../items';
  import { previewOff, previewOn } from '../preview';

  let { hunk }: { hunk: Hunk } = $props();

  const focused = $derived(app.focusedHunk === hunk.id);
  const prog = $derived(hunkProgress(hunk, app.marks));
  const keys = $derived(changedKeys(hunk));
  // A side-by-side view of an added or deleted file is half empty.
  const status = $derived(app.files.get(hunk.path)?.status);
  const split = $derived(app.layout === 'split' && status !== 'added' && status !== 'deleted');

  const STEP = 20;
  const newText = $derived(texts.entries[hunk.path]?.texts?.new ?? null);
  const newLines = $derived(newText?.split('\n') ?? []);
  const lineCount = $derived(newText == null ? 0 : newText.endsWith('\n') ? newLines.length - 1 : newLines.length);
  const extra = $derived(app.expand[hunk.id] ?? { before: 0, after: 0 });
  const space = $derived(newText == null || status === 'deleted' ? { before: 0, after: 0 } : room(hunk, app.files.get(hunk.path)?.hunks ?? [], lineCount));
  const shown = $derived(extendHunk(hunk, extra, newLines));

  /** Whole items (fn, struct, impl, use group, macro…) this hunk touches that still have unaccepted lines. */
  const items = $derived.by(() => {
    const t = texts.entries[hunk.path]?.texts;
    const file = app.files.get(hunk.path);
    if (!t || t.binary || !file) return [];
    const news: number[] = [];
    const olds: number[] = [];
    let next = hunk.newStart;
    for (const l of hunk.lines) {
      if (l.newNo != null) next = l.newNo;
      if (l.kind === 'added') news.push(l.newNo!);
      if (l.kind === 'removed') {
        news.push(next);
        olds.push(l.oldNo!);
      }
      if (l.newNo != null) next = l.newNo + 1;
    }
    // Items starting inside the hunk get a button on their first line; here only the enclosing ones.
    const first = Math.min(...hunk.lines.filter((l) => l.newNo != null).map((l) => l.newNo!));
    return itemsAround(file, t, news, olds)
      .filter((it) => it.side === 'new' && it.item.start < first)
      .map((it) => ({ ...it, open: it.keys.filter((k) => app.marks.get(k) !== 'accepted') }))
      .filter((it) => it.open.length)
      .slice(0, 4);
  });

  function grow(before: number, after: number) {
    app.expand = { ...app.expand, [hunk.id]: { before: Math.min(space.before, extra.before + before), after: Math.min(space.after, extra.after + after) } };
  }

  async function wholeFunction(e: MouseEvent) {
    e.stopPropagation();
    const news = hunk.lines.map((l) => l.newNo).filter((n): n is number => n != null);
    if (!news.length) return;
    const r = await enclosingRange(hunk.path, news[0], news[news.length - 1], newLines);
    if (!r) return app.toast('No enclosing function found');
    grow(Math.max(0, news[0] - r[0] - extra.before), Math.max(0, r[1] - news[news.length - 1] - extra.after));
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
<section
  class="hunk"
  class:focused
  class:done={prog.pending === 0}
  id="hunk-{hunk.id}"
  onclick={() => (app.focusedHunk = hunk.id)}
  onmouseenter={() => (app.hoverPath = hunk.path)}
  onmouseleave={() => app.hoverPath === hunk.path && (app.hoverPath = null)}
>
  <header class="hunk-head">
    <span class="hdr">@@ -{hunk.oldStart},{hunk.oldLen} +{hunk.newStart},{hunk.newLen} @@</span>
    {#if hunk.baseId}<span class="badge" title="The AI split a larger hunk; the rest is in another step">part of hunk</span>{/if}
    <span class="prog">{prog.accepted}/{prog.total} ✓{#if prog.deferred} · {prog.deferred} ⏸{/if}</span>
    <span class="spacer"></span>
    {#if prog.accepted < prog.total}
      <button onmouseenter={() => previewOn(keys)} onmouseleave={previewOff} onclick={(e) => { e.stopPropagation(); focusHunk(hunk.id); markKeys(keys, 'accepted'); }}>Accept hunk</button>
      <button onclick={(e) => { e.stopPropagation(); markKeys(keys, 'deferred'); }}>Later</button>
    {/if}
    {#if space.before + space.after > 0}<button class="ghost" onclick={wholeFunction} title="Show the whole enclosing function (rust-analyzer, or a guess for other languages)">⤢ Function</button>{/if}
    {#if status !== 'deleted'}<button class="ghost" onclick={(e) => { e.stopPropagation(); editFile(hunk.path, hunk.newStart); }} title="Open the file at this place; you can edit it there (e)">Open</button>{/if}
    {#if prog.accepted + prog.deferred > 0}
      <button class="ghost" onclick={(e) => { e.stopPropagation(); markKeys(keys, null, false); }} title="Back to pending">↺ {prog.accepted === prog.total ? 'Unaccept' : 'Reset'}</button>
    {/if}
  </header>
  {#if items.length}
    <div class="item-chips">
      <span class="muted">Inside:</span>
      {#each items as it}
        <button
          class="chip item-chip"
          onmouseenter={() => previewOn(it.open)}
          onmouseleave={previewOff}
          onclick={(e) => { e.stopPropagation(); markKeys(it.open, 'accepted', false); }}
          title="Accept all {it.open.length} unaccepted changed line(s) of {itemLabel(it.item)}{it.side === 'old' ? ' (removed code)' : ''}, lines {it.item.start}–{it.item.end}"
        >✓ {it.side === 'old' ? 'removed ' : ''}{itemLabel(it.item)}</button>
      {/each}
    </div>
  {/if}
  {#if extra.before < space.before}
    <div class="expand-row"><button onclick={(e) => { e.stopPropagation(); grow(STEP, 0); }}>↑ {Math.min(STEP, space.before - extra.before)} more lines</button></div>
  {/if}
  {#if split}
    <div class="rows split">
      {#each splitRows(shown) as row, i (i)}
        <div class="srow">
          <LineCell cell={row.left} path={hunk.path} />
          <LineCell cell={row.right} path={hunk.path} />
        </div>
      {/each}
    </div>
  {:else}
    <div class="rows unified">
      {#each unifiedRows(shown) as row, i (i)}
        <LineCell cell={row.cell} path={hunk.path} oldNo={row.oldNo} newNo={row.newNo} unified />
      {/each}
    </div>
  {/if}
  {#if extra.after < space.after}
    <div class="expand-row"><button onclick={(e) => { e.stopPropagation(); grow(0, STEP); }}>↓ {Math.min(STEP, space.after - extra.after)} more lines</button></div>
  {/if}
</section>
