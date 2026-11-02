<script lang="ts">
  import { api } from '../api';
  import { app } from '../state.svelte';
  import type { FileDiff } from '../types';
  import ChangeActions from './ChangeActions.svelte';
  import ChangeFile from './ChangeFile.svelte';

  /** What one AI job changed, by itself (not the whole change under review), to keep or undo. */
  let { id }: { id: string } = $props();

  const change = $derived(app.snap?.changes.find((c) => c.id === id));
  let files = $state<FileDiff[] | null>(null);
  let error = $state('');
  $effect(() => {
    const target = id;
    files = null;
    api.changeDiff(target).then(
      (f) => target === id && (files = f),
      (e) => (error = String(e)),
    );
  });
</script>

<div class="chunk-view">
  <header class="chunk-head">
    <div class="title-row">
      <h1><span class="muted feature">AI change ›</span> {change?.title ?? id}</h1>
      {#if change?.status !== 'pending'}<span class="stamp">{change?.status}</span>{/if}
    </div>
    <p class="summary muted">By {change?.by}. Only what this job changed, not the whole change under review. <strong>Accept</strong> keeps it and counts the lines it wrote as reviewed; <strong>Reject</strong> puts the files back as they were before it.</p>
    {#if change}<div class="head-actions"><span class="muted">{change.files.length} file(s)</span><span class="spacer"></span><ChangeActions {change} /></div>{/if}
  </header>
  {#if error}<p class="error">{error}</p>{/if}
  {#if files}
    {#each files as f (f.path)}<ChangeFile file={f} />{/each}
  {:else if !error}
    <p class="empty muted"><span class="spinner"></span></p>
  {/if}
</div>
