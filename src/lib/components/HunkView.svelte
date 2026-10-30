<script lang="ts">
  import { editFile, focusHunk, markKeys } from '../actions';
  import { changedKeys } from '../keys';
  import { hunkProgress } from '../progress';
  import { splitRows, unifiedRows } from '../rows';
  import { app } from '../state.svelte';
  import type { Hunk } from '../types';
  import LineCell from './LineCell.svelte';

  let { hunk }: { hunk: Hunk } = $props();

  const focused = $derived(app.focusedHunk === hunk.id);
  const prog = $derived(hunkProgress(hunk, app.marks));
  const keys = $derived(changedKeys(hunk));
  // A side-by-side view of an added or deleted file is half empty.
  const status = $derived(app.files.get(hunk.path)?.status);
  const split = $derived(app.layout === 'split' && status !== 'added' && status !== 'deleted');
</script>

<!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
<section
  class="hunk"
  class:focused
  class:done={prog.pending === 0}
  id="hunk-{hunk.id}"
  onclick={() => (app.focusedHunk = hunk.id)}
  onmouseenter={() => (app.hoverPath = hunk.path)}
  onmouseleave={() => app.hoverPath === hunk.path && (app.hoverPath = null)}
>
  <header class="hunk-head">
    <span class="hdr">@@ -{hunk.oldStart},{hunk.oldLen} +{hunk.newStart},{hunk.newLen} @@</span>
    {#if hunk.baseId}<span class="badge" title="The AI split a larger hunk; the rest is in another step">part of hunk</span>{/if}
    <span class="prog">{prog.accepted}/{prog.total} ✓{#if prog.deferred} · {prog.deferred} ⏸{/if}</span>
    <span class="spacer"></span>
    <button onclick={(e) => { e.stopPropagation(); focusHunk(hunk.id); markKeys(keys, 'accepted'); }}>Accept hunk</button>
    <button onclick={(e) => { e.stopPropagation(); markKeys(keys, 'deferred'); }}>Later</button>
    {#if status !== 'deleted'}<button class="ghost" onclick={(e) => { e.stopPropagation(); editFile(hunk.path, hunk.newStart); }} title="Open the file at this place; you can edit it there (e)">Open</button>{/if}
    {#if prog.accepted + prog.deferred > 0}
      <button class="ghost" onclick={(e) => { e.stopPropagation(); markKeys(keys, null); }}>Reset</button>
    {/if}
  </header>
  {#if split}
    <div class="rows split">
      {#each splitRows(hunk) as row, i (i)}
        <div class="srow">
          <LineCell cell={row.left} path={hunk.path} />
          <LineCell cell={row.right} path={hunk.path} />
        </div>
      {/each}
    </div>
  {:else}
    <div class="rows unified">
      {#each unifiedRows(hunk) as row, i (i)}
        <LineCell cell={row.cell} path={hunk.path} oldNo={row.oldNo} newNo={row.newNo} unified />
      {/each}
    </div>
  {/if}
</section>
