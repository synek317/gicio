<script lang="ts">
  import { go, revealThread, scrollToLine } from '../actions';
  import { parseKey } from '../keys';
  import { app } from '../state.svelte';

  /** Deferred lines collapsed into ranges per file and side. */
  const ranges = $derived.by(() => {
    const keys = [...app.marks].filter(([, s]) => s === 'deferred').map(([k]) => parseKey(k));
    keys.sort((a, b) => a.path.localeCompare(b.path) || a.side.localeCompare(b.side) || a.line - b.line);
    const out: { path: string; side: 'old' | 'new'; start: number; end: number }[] = [];
    for (const k of keys) {
      const last = out[out.length - 1];
      if (last && last.path === k.path && last.side === k.side && k.line <= last.end + 1) last.end = k.line;
      else out.push({ path: k.path, side: k.side, start: k.line, end: k.line });
    }
    return out;
  });
  const threads = $derived(app.snap?.threads.filter((t) => t.status === 'deferred') ?? []);

  function open(r: { path: string; side: 'old' | 'new'; start: number }) {
    go({ kind: 'later' });
    setTimeout(() => scrollToLine(r.path, r.side, r.start), 60);
  }
</script>

<div class="later">
  <button class="primary wide" disabled={!ranges.length} onclick={() => go({ kind: 'later' })}>Review all later items</button>
  <h3>Lines ({ranges.length} ranges)</h3>
  <ul>
    {#each ranges as r}
      <li><button class="link" onclick={() => open(r)}>{r.path}<span class="muted">:{r.side === 'old' ? '−' : '+'}{r.start}{r.end > r.start ? `–${r.end}` : ''}</span></button></li>
    {:else}
      <li class="muted">Nothing put off.</li>
    {/each}
  </ul>
  <h3>Discussions ({threads.length})</h3>
  <ul>
    {#each threads as t}
      <li><button class="link" onclick={() => revealThread(t)}>{t.messages[0]?.body.slice(0, 80) ?? '(empty)'}</button></li>
    {:else}
      <li class="muted">No discussions put off.</li>
    {/each}
  </ul>
</div>
