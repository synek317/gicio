<script lang="ts">
  import type { JobInfo } from '../types';

  /** `compact` shows only the latest line until clicked. */
  let { job, lines = 8, compact = false }: { job: JobInfo; lines?: number; compact?: boolean } = $props();
  let pre: HTMLElement | undefined = $state();
  let expanded = $state(false);

  $effect(() => {
    void job.log.length;
    if (pre) pre.scrollTop = pre.scrollHeight;
  });
</script>

{#if compact && !expanded}
  <button class="job-last" onclick={(e) => { e.stopPropagation(); expanded = true; }} title="Show the full log">{job.log.at(-1) ?? 'waiting for output…'}</button>
{:else}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
  <pre class="job-log" bind:this={pre} style="max-height:{lines * 1.5}em" onclick={(e) => { if (compact) { e.stopPropagation(); expanded = false; } }}>{job.log.length ? job.log.join('\n') : 'waiting for output…'}</pre>
{/if}
