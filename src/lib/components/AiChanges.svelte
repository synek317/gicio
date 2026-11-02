<script lang="ts">
  import { go } from '../actions';
  import { app } from '../state.svelte';
  import ChangeActions from './ChangeActions.svelte';

  /** AI changes still waiting for a decision, above the steps. */
</script>

{#if app.pendingChanges.length}
  <h3 class="qhead">AI changes to check <span class="muted">({app.pendingChanges.length})</span></h3>
  <ul class="queue ai-changes">
    {#each app.pendingChanges as c (c.id)}
      <li class="qrow">
        <button class="qitem" class:active={app.view?.kind === 'change' && app.view.id === c.id} onclick={() => go({ kind: 'change', id: c.id })} title="Show what the AI changed">
          <span class="qicon">✎</span>
          <span class="qtext">
            <span class="qtitle">{c.title}</span>
            <span class="qmeta">{c.by} · {c.files.length} file(s)</span>
          </span>
        </button>
        <div class="change-row-actions"><ChangeActions change={c} small /></div>
      </li>
    {/each}
  </ul>
{/if}
