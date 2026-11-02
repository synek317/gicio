<script lang="ts">
  import { app } from '../state.svelte';
  import FileTree from './FileTree.svelte';
  import QueueList from './QueueList.svelte';

  const tabs = [
    ['steps', 'Steps'],
    ['files', 'Files'],
  ] as const;
</script>

<aside class="sidebar">
  <nav class="tabs">
    {#each tabs as [id, label]}
      <button class:active={app.sidebarTab === id} onclick={() => (app.sidebarTab = id)}>{label}</button>
    {/each}
  </nav>
  <div class="side-body">
    {#if app.sidebarTab === 'steps'}<QueueList />{:else}<FileTree />{/if}
  </div>
</aside>
