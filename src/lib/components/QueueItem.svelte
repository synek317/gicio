<script lang="ts">
  import { go, markKeys } from '../actions';
  import { changedKeys } from '../keys';
  import { progressState } from '../progress';
  import { progressIcons, progressTooltip } from '../progress-icon';
  import { app } from '../state.svelte';
  import type { Chunk } from '../types';
  import ProgressBar from './ProgressBar.svelte';
  import StepFiles from './StepFiles.svelte';

  let { chunk: c, label }: { chunk: Chunk; label: string } = $props();

  const p = $derived(app.chunkProgress(c));
  const st = $derived(progressState(p));
  const threads = $derived(app.snap?.threads.filter((t) => t.chunk === c.id && t.status === 'open').length ?? 0);
  const active = $derived(app.view?.kind === 'chunk' && app.view.id === c.id);

  let item: HTMLElement | undefined = $state();
  $effect(() => {
    if (active) item?.scrollIntoView({ block: 'nearest' });
  });

  function unaccept(e: MouseEvent) {
    e.stopPropagation();
    const keys = app.chunkHunks(c).flatMap(changedKeys).filter((k) => app.marks.get(k) === 'accepted');
    markKeys(keys, null, false);
  }
</script>

<li class="qrow" bind:this={item}>
  {#if p.accepted}
    <button class="ghost small unaccept" onclick={unaccept} title="Unaccept this whole step (its lines become pending again)">↺</button>
  {/if}
  <button class="qitem {st}" class:active onclick={() => go({ kind: 'chunk', id: c.id })}>
    <span class="qicon" title={progressTooltip(st, p)}>{progressIcons[st]}</span>
    <span class="qtext">
      <span class="qtitle">{label}. {c.title}</span>
      {#if st !== 'done'}
        <span class="qmeta">{p.total} lines{#if threads} · 💬 {threads}{/if}{#if c.notes?.length} · ? {c.notes.length}{/if}{#if c.origin.kind === 'followUp'} · follow-up{/if}</span>
        <ProgressBar {p} thin />
      {/if}
    </span>
    {#if st === 'done'}<span class="stamp">Done</span>{/if}
  </button>
  {#if active}<StepFiles chunk={c} />{/if}
</li>
