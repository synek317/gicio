<script lang="ts">
  import ChangeView from './ChangeView.svelte';
  import ChunkView from './ChunkView.svelte';
  import FindBar from './FindBar.svelte';
  import FileView from './FileView.svelte';
  import PanelResizer from './PanelResizer.svelte';
  import RefsView from './RefsView.svelte';
  import Sidebar from './Sidebar.svelte';
  import StatusBar from './StatusBar.svelte';
  import ThreadsPanel from './ThreadsPanel.svelte';
  import TopBar from './TopBar.svelte';
  import { find } from '../find.svelte';
  import { panelColumn, panelPref } from '../panels.svelte';
  import { app } from '../state.svelte';

  const v = $derived(app.view);

  // Offer to wrap up when the last change gets accepted (not when an already finished review opens).
  let wasFinished = app.finished;
  $effect(() => {
    const finished = app.finished;
    if (finished && !wasFinished) app.showFinish = true;
    wasFinished = finished;
  });
</script>

<div class="app layout-{app.layout}" style:--left-panel={panelColumn('left')} style:--right-panel={panelColumn('right')}>
  <TopBar />
  <Sidebar />
  {#if panelPref('left').visible}<PanelResizer side="left" />{/if}
  <main id="diff-scroll" class="main" class:no-scroll={v?.kind === 'source'}>
    {#if v?.kind === 'source'}
      {#key `${v.absPath}#${app.fileRev}`}<FileView absPath={v.absPath} repoPath={v.repoPath} line={v.line} col={v.col ?? null} len={v.len ?? 0} title={v.title} />{/key}
    {:else if v?.kind === 'change'}
      <ChangeView id={v.id} />
    {:else if v?.kind === 'refs'}
      <RefsView title={v.title} locs={v.locs} />
    {:else if app.snap?.files.length}
      <ChunkView />
    {:else}
      <div class="empty"><h2>No changes</h2><p class="muted">The working tree matches the base revision.</p></div>
    {/if}
  </main>
  {#if find.open}<FindBar />{/if}
  <ThreadsPanel />
  {#if panelPref('right').visible}<PanelResizer side="right" />{/if}
  <StatusBar />
</div>
