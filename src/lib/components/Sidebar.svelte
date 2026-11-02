<script lang="ts">
  import { app } from '../state.svelte';
  import FileTree from './FileTree.svelte';
  import QueueList from './QueueList.svelte';
  import SearchPanel from './SearchPanel.svelte';

  const tabs = [
    ['steps', 'Steps'],
    ['files', 'Files'],
    ['search', 'Search'],
  ] as const;
</script>

<aside class="sidebar">
  <nav class="tabs">
    {#each tabs as [id, label]}
      <button class:active={app.sidebarTab === id} onclick={() => (app.sidebarTab = id)}>{label}</button>
    {/each}
  </nav>
  <div class="side-body">
    {#if app.sidebarTab === 'steps'}<QueueList />{:else if app.sidebarTab === 'files'}<FileTree />{:else}<SearchPanel />{/if}
  </div>
</aside>
