<script lang="ts">
  import { go } from '../actions';
  import { progressState } from '../progress';
  import { app } from '../state.svelte';
  import type { Chunk } from '../types';
  import ProgressBar from './ProgressBar.svelte';

  let { chunk: c, index }: { chunk: Chunk; index: number } = $props();

  const icons = { todo: '○', started: '◐', deferred: '⏸', done: '✓' };
  const p = $derived(app.chunkProgress(c));
  const st = $derived(progressState(p));
  const threads = $derived(app.snap?.threads.filter((t) => t.chunk === c.id && t.status === 'open').length ?? 0);
  const active = $derived(app.view?.kind === 'chunk' && app.view.id === c.id);
</script>

<li>
  <button class="qitem {st}" class:active onclick={() => go({ kind: 'chunk', id: c.id })}>
    <span class="qicon">{icons[st]}</span>
    <span class="qtext">
      <span class="qtitle">{index + 1}. {c.title}</span>
      <span class="qmeta">{p.total} lines{#if threads} · 💬 {threads}{/if}{#if c.notes?.length} · ? {c.notes.length}{/if}{#if c.origin.kind === 'followUp'} · follow-up{/if}</span>
      {#if st !== 'done'}<ProgressBar {p} thin />{/if}
    </span>
  </button>
</li>
