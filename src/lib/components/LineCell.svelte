<script lang="ts">
  import { definitionAtPointer, hoverCode, leaveCode } from '../lsp';
  import { texts } from '../texts.svelte';
  import { app } from '../state.svelte';
  import { revealThread } from '../actions';
  import { renderMarkdown } from '../markdown';
  import { settings } from '../app-settings.svelte';
  import { openMenu } from '../menu';
  import type { Cell } from '../rows';
  import CodeText from './CodeText.svelte';
  import Gutter from './Gutter.svelte';
  import FindingCard from './FindingCard.svelte';
  import { itemStarts } from '../item-keys';
  import { itemLabel } from '../items';
  import { markKeys } from '../actions';
  import { previewOff, previewOn } from '../preview';

  let { cell, path, oldNo = null, newNo = null, unified = false }: { cell: Cell | null; path: string; oldNo?: number | null; newNo?: number | null; unified?: boolean } = $props();

  const status = $derived(cell?.key ? app.marks.get(cell.key) : undefined);
  const selected = $derived(!!cell?.key && app.selSet.has(cell.key));
  const threads = $derived(cell ? (app.threadLines.get(`${path}|${cell.side}|${cell.no}`) ?? []) : []);
  const notes = $derived(cell ? (app.noteLines.get(`${path}|${cell.side}|${cell.no}`) ?? []) : []);
  let showNotes = $state(settings.value.prefs.notesExpanded);
  /** Items (fn, struct, use group…) starting on this line, with their unaccepted changed lines. */
  const startsHere = $derived.by(() => {
    if (!cell) return [];
    const t = texts.entries[path]?.texts;
    const file = app.files.get(path);
    if (!t || t.binary || !file) return [];
    return (itemStarts(file, t).get(`${cell.side}|${cell.no}`) ?? [])
      .map((it) => ({ ...it, open: it.keys.filter((k) => app.marks.get(k) !== 'accepted') }))
      .filter((it) => it.open.length);
  });
  const findings = $derived(cell ? (app.findingLines.get(`${path}|${cell.side}|${cell.no}`) ?? []) : []);
  // A finding box is shown once, on its first line.
  const findingsHere = $derived(findings.filter((f) => f.line === cell?.no));

  function click(e: MouseEvent) {
    if (cell && (e.metaKey || e.ctrlKey) && cell.side === 'new') {
      e.preventDefault();
      definitionAtPointer(e, path, cell.no);
    }
  }
</script>

{#if cell}
  <div
    class="line {cell.kind} {status ?? ''}"
    class:has-finding={findings.length > 0}
    class:preview={!!cell.key && app.preview.has(cell.key)}
    class:selected
    data-line="{path}|{cell.side}|{cell.no}"
  >
    <Gutter {cell} {oldNo} {newNo} {status} {unified} />
    <span class="tmark">
      {#if findingsHere.length}
        <button class="finding-dot sev-{findingsHere[0].severity}" class:on={showNotes} title={findingsHere.map((f) => f.title).join('\n')} onclick={() => (showNotes = !showNotes)}>!</button>
      {:else if notes.length}
        <button class="note-dot" class:on={showNotes} title={notes.map((n) => n.text).join('\n\n')} onclick={() => (showNotes = !showNotes)}>?</button>
      {:else if threads.length}
        <button class="thread-dot" class:open={threads.some((t) => t.status === 'open')} title="{threads.length} thread(s)" onclick={() => revealThread(threads[0])}>💬</button>
      {/if}
    </span>
    <!-- svelte-ignore a11y_no_noninteractive_element_interactions, a11y_click_events_have_key_events -->
    <code
      class="code"
      onmousemove={(e) => cell.side === 'new' && hoverCode(e, path, cell.no)}
      onmouseleave={leaveCode}
      onclick={click}
      oncontextmenu={(e) => openMenu(e, path, cell.side, cell.no)}
    ><CodeText tokens={texts.tokens(path, cell.side, cell.no)} text={cell.text} hl={cell.hl} />{#each startsHere as it}<button
          class="item-chip inline"
          onclick={(e) => { e.stopPropagation(); markKeys(it.open, 'accepted', false); }}
          onmouseenter={() => previewOn(it.open)}
          onmouseleave={previewOff}
          title="Accept all {it.open.length} unaccepted changed line(s) of {itemLabel(it.item)}, lines {it.item.start}–{it.item.end}"
        >✓ {itemLabel(it.item)}</button>{/each}</code>
    {#if showNotes && (notes.length || findingsHere.length)}
      <div class="note-box md">
        {#each findingsHere as f (f.id)}<FindingCard finding={f} compact />{/each}
        {#each notes as n}<div>{@html renderMarkdown(n.text)}</div>{/each}
      </div>
    {/if}
  </div>
{:else}
  <div class="line filler"><span class="gutter"></span><span class="tmark"></span><code class="code"></code></div>
{/if}
