<script lang="ts">
  import { openHit, search } from '../project-search.svelte';
  import { app } from '../state.svelte';
  import type { FileHits, LineHit } from '../types';

  /** One file of the project search results, with its matching lines. */
  let { file }: { file: FileHits } = $props();

  const collapsed = $derived(!!search.collapsed[file.path]);
  const count = $derived(file.lines.reduce((n, l) => n + l.matches, 0));
  const name = $derived(file.path.split('/').pop());
  const dir = $derived(file.path.split('/').slice(0, -1).join('/'));
  /** Files in the change under review carry their status, like in the file tree. */
  const status = $derived(app.files.get(file.path)?.status);

  /** The preview cut into plain text and matches. */
  function pieces(hit: LineHit): { text: string; on: boolean }[] {
    const out: { text: string; on: boolean }[] = [];
    let at = 0;
    for (const [from, to] of hit.ranges) {
      if (from > at) out.push({ text: hit.preview.slice(at, from), on: false });
      out.push({ text: hit.preview.slice(from, to), on: true });
      at = to;
    }
    if (at < hit.preview.length) out.push({ text: hit.preview.slice(at), on: false });
    return out;
  }
</script>

<div class="search-file">
  <button class="search-file-head" onclick={() => (search.collapsed = { ...search.collapsed, [file.path]: !collapsed })} title={file.path}>
    <span class="chev">{collapsed ? '▸' : '▾'}</span>
    <span class="sname">{name}</span>
    <span class="sdir">{dir}</span>
    {#if status}<span class="status s-{status}" title="Part of the change under review">{status[0]}</span>{/if}
    <span class="scount">{count}</span>
  </button>
  {#if !collapsed}
    {#each file.lines as hit (hit.line)}
      <button class="search-hit" class:active={search.active === `${file.path}:${hit.line}`} onclick={() => openHit(file, hit)} title="{file.path}:{hit.line}">
        <span class="no">{hit.line}</span>
        <code>{#each pieces(hit) as p}{#if p.on}<mark>{p.text}</mark>{:else}{p.text}{/if}{/each}</code>
      </button>
    {/each}
  {/if}
</div>
