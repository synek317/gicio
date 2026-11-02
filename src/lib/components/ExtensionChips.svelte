<script lang="ts">
  import { search } from '../project-search.svelte';
  import type { FileHits } from '../types';

  /** File types among the results; a click narrows the search to one ("files to include"), a second
   *  click shows all again. */
  let { files }: { files: FileHits[] } = $props();

  const extOf = (path: string) => {
    const name = path.split('/').pop() ?? '';
    const i = name.lastIndexOf('.');
    return i > 0 ? name.slice(i + 1) : '';
  };

  const exts = $derived.by(() => {
    const m = new Map<string, number>();
    for (const f of files) {
      const e = extOf(f.path);
      if (e) m.set(e, (m.get(e) ?? 0) + f.lines.reduce((n, l) => n + l.matches, 0));
    }
    return [...m].sort((a, b) => b[1] - a[1]);
  });
  const only = $derived(/^\*\.([^\s,/*]+)$/.exec(search.include.trim())?.[1] ?? null);

  function pick(ext: string) {
    search.include = only === ext ? '' : `*.${ext}`;
  }
</script>

{#if exts.length > 1 || only}
  <div class="ext-chips">
    {#each exts as [ext, n]}
      <button class="chip" class:on={only === ext} onclick={() => pick(ext)} title={only === ext ? 'Search all file types again' : `Only *.${ext} files`}>.{ext} <span class="muted">{n}</span></button>
    {/each}
  </div>
{/if}
