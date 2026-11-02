<script lang="ts">
  import { acceptChange, rejectChange } from '../change-actions';
  import type { AgentChange } from '../types';

  /** Keep or undo an AI change; once decided, what was decided. */
  let { change, small = false }: { change: AgentChange; small?: boolean } = $props();
</script>

{#if change.status === 'pending'}
  <button class="primary-soft" class:small onclick={(e) => (e.stopPropagation(), acceptChange(change.id))} title="Keep it: the lines the AI wrote count as reviewed">✓ Accept</button>
  <button class="danger" class:small onclick={(e) => (e.stopPropagation(), rejectChange(change.id))} title="Undo it: the files go back to how they were before">✕ Reject</button>
{:else}
  <span class="badge st-{change.status === 'accepted' ? 'resolved' : 'outdated'}">{change.status}</span>
{/if}
