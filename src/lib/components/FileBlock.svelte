<script lang="ts">
  import { editFile, markKeys } from '../actions';
  import { blockKey, jumpFile, setCollapsed } from '../files-nav';
  import { changedKeys } from '../keys';
  import { hunksProgress } from '../progress';
  import { app } from '../state.svelte';
  import { texts } from '../texts.svelte';
  import type { Hunk } from '../types';
  import HunkView from './HunkView.svelte';
  import { previewOff, previewOn } from '../preview';

  let { path, hunks }: { path: string; hunks: Hunk[] } = $props();

  const file = $derived(app.files.get(path));
  const prog = $derived(hunksProgress(hunks, app.marks));
  const keys = $derived(hunks.flatMap(changedKeys));
  const unaccepted = $derived(keys.filter((k) => app.marks.get(k) !== 'accepted'));
  const accepted = $derived(keys.filter((k) => app.marks.get(k) === 'accepted'));
  const fileTotal = $derived(file ? hunksProgress(file.hunks, app.marks).total : prog.total);
  const whole = $derived(prog.total >= fileTotal);
  const collapsed = $derived(!!app.collapsed[blockKey(path)]);
  const done = $derived(prog.total > 0 && prog.pending === 0 && prog.deferred === 0);
  let block: HTMLElement | undefined = $state();

  $effect(() => {
    if (file) texts.ensure(file);
  });

  async function accept() {
    await markKeys(unaccepted, 'accepted');
    setCollapsed(path, true);
  }

  async function unaccept() {
    await markKeys(accepted, null, false);
    setCollapsed(path, false);
  }
</script>

<div class="file-block" class:collapsed class:done bind:this={block} data-path={path}>
  <header class="file-head">
    <button class="ghost chevron" onclick={() => setCollapsed(path, !collapsed)} title="Collapse / expand (x)">{collapsed ? '▸' : '▾'}</button>
    <span class="status s-{file?.status}">{file?.status}</span>
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <span class="path" onclick={() => setCollapsed(path, !collapsed)}>{#if file?.oldPath}<span class="muted">{file.oldPath} → </span>{/if}{path}</span>
    <span class="prog">{#if done}✓ {/if}{prog.accepted + prog.deferred}/{prog.total}{#if !whole} <span class="muted">· {fileTotal} in file</span>{/if}</span>
    <span class="spacer"></span>
    {#if file?.status !== 'deleted'}
      <button class="ghost" onclick={() => editFile(path, hunks[0]?.newStart ?? 1)} title="Open the whole file as it is now; you can edit it there">Open file</button>
    {/if}
    {#if unaccepted.length}
      <button onclick={() => markKeys(unaccepted, 'deferred')} title="Put these changes off for later">Later</button>
      <button class="primary-soft" onmouseenter={() => previewOn(unaccepted)} onmouseleave={previewOff} onclick={accept} title={whole ? 'Accept all changes in this file' : 'Accept the changes of this file shown in this step; the rest of the file is in other steps'}>
        ✓ {whole ? 'Accept file' : `Accept this part (${prog.total} lines)`}
      </button>
    {/if}
    {#if accepted.length}
      <button class="ghost" onclick={unaccept} title="Take back the acceptance of these lines (they become pending again)">↺ Unaccept{unaccepted.length ? ` (${accepted.length})` : ''}</button>
    {/if}
    <button class="ghost" onclick={() => jumpFile(1)} title="Next file (f)">↓</button>
  </header>
  {#if !collapsed}
    {#each hunks as hunk (hunk.id)}
      <HunkView {hunk} />
    {/each}
  {:else}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="collapsed-row muted small" onclick={() => setCollapsed(path, false)}>{hunks.length} hunk(s) hidden, click to show</div>
  {/if}
</div>
