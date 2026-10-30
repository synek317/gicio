<script lang="ts">
  import { app } from '../state.svelte';
  import FileTree from './FileTree.svelte';
  import LaterList from './LaterList.svelte';
  import QueueList from './QueueList.svelte';

  const tabs = [
    ['queue', 'Queue'],
    ['files', 'Files'],
    ['later', 'Later'],
  ] as const;
</script>

<aside class="sidebar">
  <nav class="tabs">
    {#each tabs as [id, label]}
      <button class:active={app.sidebarTab === id} onclick={() => (app.sidebarTab = id)}>
        {label}{#if id === 'later' && app.overall.deferred} <span class="count">{app.overall.deferred}</span>{/if}
      </button>
    {/each}
  </nav>
  <div class="side-body">
    {#if app.sidebarTab === 'queue'}<QueueList />{:else if app.sidebarTab === 'files'}<FileTree />{:else}<LaterList />{/if}
  </div>
</aside>
