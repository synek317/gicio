<script lang="ts">
  import type { JobInfo } from '../types';

  let { job, lines = 8 }: { job: JobInfo; lines?: number } = $props();
  let pre: HTMLElement | undefined = $state();

  $effect(() => {
    void job.log.length;
    if (pre) pre.scrollTop = pre.scrollHeight;
  });
</script>

<pre class="job-log" bind:this={pre} style="max-height:{lines * 1.5}em">{job.log.length ? job.log.join('\n') : 'waiting for output…'}</pre>
