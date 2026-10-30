<script lang="ts">
  import { editFile, markKeys } from '../actions';
  import { changedKeys } from '../keys';
  import { hunksProgress } from '../progress';
  import { app } from '../state.svelte';
  import { texts } from '../texts.svelte';
  import type { Hunk } from '../types';
  import HunkView from './HunkView.svelte';

  let { path, hunks }: { path: string; hunks: Hunk[] } = $props();

  const file = $derived(app.files.get(path));
  const prog = $derived(hunksProgress(hunks, app.marks));
  const keys = $derived(hunks.flatMap(changedKeys));
  const fileTotal = $derived(file ? hunksProgress(file.hunks, app.marks).total : prog.total);
  const whole = $derived(prog.total >= fileTotal);
  let block: HTMLElement | undefined = $state();

  $effect(() => {
    if (file) texts.ensure(file);
  });

  // Track whether this block is on screen, so the file tree can highlight it.
  $effect(() => {
    if (!block) return;
    const p = path;
    let visible = false;
    const set = (on: boolean) => {
      if (on === visible) return;
      visible = on;
      const n = (app.onScreen[p] ?? 0) + (on ? 1 : -1);
      const next = { ...app.onScreen };
      if (n > 0) next[p] = n;
      else delete next[p];
      app.onScreen = next;
    };
    const io = new IntersectionObserver(([e]) => set(e.isIntersecting), { root: document.getElementById('diff-scroll') });
    io.observe(block);
    return () => {
      io.disconnect();
      set(false);
    };
  });
</script>

<div class="file-block" bind:this={block}>
  <header class="file-head">
    <span class="status s-{file?.status}">{file?.status}</span>
    <span class="path">{#if file?.oldPath}<span class="muted">{file.oldPath} → </span>{/if}{path}</span>
    <span class="prog">{prog.accepted + prog.deferred}/{prog.total}{#if !whole} <span class="muted">· {fileTotal} in file</span>{/if}</span>
    <span class="spacer"></span>
    {#if file?.status !== 'deleted'}
      <button class="ghost" onclick={() => editFile(path, hunks[0]?.newStart ?? 1)} title="Open the whole file as it is now; you can edit it there">Open file</button>
    {/if}
    <button onclick={() => markKeys(keys, 'deferred')} title="Put these changes off for later">Later</button>
    <button class="primary-soft" onclick={() => markKeys(keys, 'accepted')} title={whole ? 'Accept all changes in this file' : 'Accept the changes of this file shown in this step; the rest of the file is in other steps'}>
      ✓ {whole ? 'Accept file' : `Accept this part (${prog.total} lines)`}
    </button>
  </header>
  {#each hunks as hunk (hunk.id)}
    <HunkView {hunk} />
  {/each}
</div>
