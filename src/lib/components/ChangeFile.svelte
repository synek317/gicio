<script lang="ts">
  import { unifiedRows } from '../rows';
  import type { FileDiff } from '../types';
  import CodeText from './CodeText.svelte';

  /** One file of an AI change, read only: what the AI removed and wrote. */
  let { file }: { file: FileDiff } = $props();
</script>

<div class="file-block">
  <header class="file-head">
    <span class="status s-{file.status}">{file.status}</span>
    <span class="path">{file.path}</span>
  </header>
  {#each file.hunks as hunk (hunk.id)}
    <section class="hunk">
      <header class="hunk-head"><span class="hdr">@@ -{hunk.oldStart},{hunk.oldLen} +{hunk.newStart},{hunk.newLen} @@</span></header>
      <div class="rows unified">
        {#each unifiedRows(hunk) as row, i (i)}
          <div class="line {row.cell.kind}">
            <span class="gutter"><span class="no">{row.oldNo ?? ''}</span><span class="no">{row.newNo ?? ''}</span><span class="mk">{row.cell.kind === 'added' ? '+' : row.cell.kind === 'removed' ? '−' : ''}</span></span>
            <span class="tmark"></span>
            <code class="code"><CodeText tokens={null} text={row.cell.text} hl={row.cell.hl} /></code>
          </div>
        {/each}
      </div>
    </section>
  {:else}
    <p class="muted small change-empty">No text changes (binary file, or only its mode changed).</p>
  {/each}
</div>
