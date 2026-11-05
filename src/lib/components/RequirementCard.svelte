<script lang="ts">
  import { run } from '../actions';
  import { api } from '../api';
  import { discussRequirement, implementRequirements, needsWork, showCode } from '../intent-actions';
  import type { Requirement, Verdict } from '../types';

  let { requirement: r }: { requirement: Requirement } = $props();

  const verdicts: Record<Verdict, { icon: string; label: string }> = {
    done: { icon: '✅', label: 'Done' },
    partial: { icon: '⚠️', label: 'Partly done' },
    missing: { icon: '❌', label: 'Missing' },
    unclear: { icon: '❓', label: 'Cannot tell from the code' },
  };
  const setStatus = (status: Requirement['status']) => run(api.setRequirementStatus([r.id], status));
</script>

<div class="card requirement" class:dismissed={r.status === 'dismissed'}>
  <div class="row req-head">
    <span class="sev" title={verdicts[r.verdict].label}>{verdicts[r.verdict].icon}</span>
    <strong class="ftitle">{r.text}</strong>
  </div>
  {#if r.status === 'implementing'}
    <span class="muted small"><span class="spinner"></span> The AI is working on it…</span>
  {:else if r.status === 'implemented'}
    <span class="badge" title="Check again to see whether it is met now">AI worked on it</span>
  {:else if r.status === 'dismissed'}
    <span class="badge">dismissed</span>
  {/if}
  {#if r.note}<p class="small req-note">{r.note}</p>{/if}
  {#if r.evidence.length}
    <div class="row wrap req-evidence">
      {#each r.evidence as e}<button class="link small" onclick={() => showCode(e)}>{e.path}:{e.line}</button>{/each}
    </div>
  {/if}
  {#if r.status !== 'implementing'}
    <div class="row">
      {#if needsWork(r)}
        <button class="small primary" onclick={() => implementRequirements([r])} title="Queue an AI job to finish this; the change lands as a step at the end of the queue">Implement</button>
      {/if}
      <button class="small" onclick={() => discussRequirement(r)}>Discuss</button>
      {#if r.status === 'dismissed'}
        <button class="small ghost" onclick={() => setStatus('open')}>Reopen</button>
      {:else if r.verdict !== 'done'}
        <button class="small ghost" onclick={() => setStatus('dismissed')} title="It does not matter for this change">Dismiss</button>
      {/if}
    </div>
  {/if}
</div>
