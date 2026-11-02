<script lang="ts">
  import { go } from '../actions';
  import { hunksProgress, progressState } from '../progress';
  import { progressIcons, progressTooltip } from '../progress-icon';
  import { app } from '../state.svelte';
  import type { Chunk } from '../types';
  import ProgressBar from './ProgressBar.svelte';
  import QueueItem from './QueueItem.svelte';

  /** A feature with its steps nested below. */
  let { title, label, steps }: { title: string; label: string; steps: { chunk: Chunk; label: string }[] } = $props();

  const p = $derived(hunksProgress(steps.flatMap((s) => app.chunkHunks(s.chunk)), app.marks));
  const st = $derived(progressState(p));

  function open() {
    const next = steps.find((s) => app.chunkProgress(s.chunk).pending > 0) ?? steps[0];
    go({ kind: 'chunk', id: next.chunk.id });
  }
</script>

<li class="qgroup">
  <button class="qitem qgroup-head {st}" onclick={open} title="Go to the first step of this feature with something left">
    <span class="qicon" title={progressTooltip(st, p)}>{progressIcons[st]}</span>
    <span class="qtext">
      <span class="qtitle">{label}. {title}</span>
      <span class="qmeta">{steps.length} steps · {p.total} lines</span>
      {#if st !== 'done'}<ProgressBar {p} thin />{/if}
    </span>
    {#if st === 'done'}<span class="stamp">Done</span>{/if}
  </button>
  <ol class="queue nested">
    {#each steps as s (s.chunk.id)}<QueueItem chunk={s.chunk} label={s.label} />{/each}
  </ol>
</li>
