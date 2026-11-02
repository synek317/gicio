<script lang="ts">
  import { scrollToFile } from '../files-nav';
  import { hunksProgress, progressState } from '../progress';
  import { progressIcons, progressTooltip } from '../progress-icon';
  import { app } from '../state.svelte';
  import type { Chunk } from '../types';

  /** The files of the selected step, right under it in the step list; a click brings one into view. */
  let { chunk }: { chunk: Chunk } = $props();

  const files = $derived.by(() => {
    const hunks = app.chunkHunks(chunk);
    return [...new Set(hunks.map((h) => h.path))].map((path) => {
      const p = hunksProgress(hunks.filter((h) => h.path === path), app.marks);
      const cut = path.lastIndexOf('/');
      return { path, dir: path.slice(0, cut + 1), name: path.slice(cut + 1), p, st: progressState(p), status: app.files.get(path)?.status };
    });
  });
</script>

{#if files.length}
  <ul class="step-files">
    {#each files as f (f.path)}
      <li>
        <button class="step-file {f.st}" class:onscreen={!!app.onScreen[f.path]} onclick={() => scrollToFile(f.path)} title={f.path}>
          <span class="qicon" title={progressTooltip(f.st, f.p)}>{progressIcons[f.st]}</span>
          <span class="status s-{f.status}">{f.status?.[0].toUpperCase()}</span>
          <span class="step-file-name"><span class="muted">{f.dir}</span>{f.name}</span>
        </button>
      </li>
    {/each}
  </ul>
{/if}
