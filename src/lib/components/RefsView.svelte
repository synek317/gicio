<script lang="ts">
  import { openLocation } from '../lsp';
  import type { SourceLocation } from '../types';

  let { title, locs }: { title: string; locs: SourceLocation[] } = $props();

  const groups = $derived.by(() => {
    const m = new Map<string, SourceLocation[]>();
    for (const l of locs) m.set(l.repoPath ?? l.absPath, [...(m.get(l.repoPath ?? l.absPath) ?? []), l]);
    return [...m];
  });
</script>

<div class="refs-view">
  <header class="chunk-head"><h1>{title}</h1><span class="muted">{locs.length} result(s) in {groups.length} file(s)</span></header>
  {#each groups as [file, items]}
    <div class="file-block">
      <header class="file-head"><span class="path">{file}</span></header>
      {#each items as l}
        <button class="ref-row" onclick={() => openLocation(l)}>
          <span class="no">{l.line + 1}</span><code>{l.preview ?? ''}</code>
        </button>
      {/each}
    </div>
  {/each}
</div>
