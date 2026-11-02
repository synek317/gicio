<script lang="ts">
  import { revealLine, run } from '../actions';
  import { api } from '../api';
  import { renderMarkdown } from '../markdown';
  import { app } from '../state.svelte';
  import type { Finding } from '../types';

  let { finding: f, compact = false }: { finding: Finding; compact?: boolean } = $props();
  let open = $state(false);

  function discuss() {
    app.composer = { anchor: { path: f.path, side: f.side, start: f.line, end: Math.max(f.line, f.endLine) }, chunk: app.view?.kind === 'chunk' ? app.view.id : null };
    app.composerDraft = `${f.title}${f.detail ? `\n\n${f.detail}` : ''}`;
    app.rightTab = 'conversations';
  }
</script>

<div class="finding sev-{f.severity} st-{f.status}">
  <div class="row">
    <span class="sev">{f.severity === 'error' ? '⛔' : f.severity === 'warning' ? '⚠️' : 'ℹ️'}</span>
    <strong class="ftitle">{f.title}</strong>
    <span class="badge">{f.source}</span>
    {#if f.status !== 'open'}<span class="badge st-{f.status}">{f.status}</span>{/if}
    {#if f.outdated}<span class="badge outdated" title="The code this finding was about has changed or been removed">outdated</span>{/if}
  </div>
  {#if !compact}
    <button class="link small" onclick={() => revealLine(f.path, f.side, f.line)}>{f.path}:{f.side === 'old' ? '−' : ''}{f.line}{f.endLine > f.line ? `–${f.endLine}` : ''}</button>
  {/if}
  {#if f.detail}
    {#if compact || open}
      <div class="md fdetail" class:mono-detail={f.source !== 'ai'}>{#if f.source !== 'ai'}<pre>{f.detail}</pre>{:else}{@html renderMarkdown(f.detail)}{/if}</div>
    {:else}
      <button class="link small" onclick={() => (open = true)}>details</button>
    {/if}
  {/if}
  {#if f.status === 'open'}
    <div class="row">
      <button class="small primary" onclick={() => run(api.fixFindings([f.id]))} title="Queue an AI fix; the change lands as a step at the end of the queue">Fix</button>
      <button class="small" onclick={discuss}>Discuss</button>
      <button class="small ghost" onclick={() => run(api.setFindingsStatus([f.id], 'dismissed'))}>Dismiss</button>
    </div>
  {:else if f.status === 'dismissed' || f.status === 'fixed'}
    <button class="small ghost" onclick={() => run(api.setFindingsStatus([f.id], 'open'))}>Reopen</button>
  {/if}
</div>
