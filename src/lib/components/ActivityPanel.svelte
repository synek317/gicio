<script lang="ts">
  import { revealThread } from '../actions';
  import { api } from '../api';
  import { app } from '../state.svelte';
  import type { JobInfo, JobKind } from '../types';
  import JobLog from './JobLog.svelte';

  const active = $derived(app.jobs.filter((j) => j.status === 'running' || j.status === 'queued'));
  const recent = $derived(app.jobs.filter((j) => j.status !== 'running' && j.status !== 'queued').slice(-5).reverse());
  let showRecent = $state(false);
  let open = $state<Record<string, boolean>>({});
  const icon: Record<JobKind, string> = { ask: '💬', todo: '📋', plan: '🗺', chat: '🗨', summary: '📝', review: '✉️', aiReview: '🔎', fix: '🛠', commit: '✔', tool: '⚙' };

  function stop(j: JobInfo) {
    api.cancelJob(j.id).catch((e) => app.toast(String(e), 'error'));
  }

  function openThread(j: JobInfo) {
    const t = app.snap?.threads.find((t) => t.id === j.thread);
    if (t) revealThread(t);
  }
</script>

{#if active.length || recent.length}
  <section class="activity">
    <header class="row">
      <h3>AI activity</h3>
      {#if active.length}<span class="spinner"></span>{/if}
      <span class="spacer"></span>
      {#if recent.length}<button class="ghost small" onclick={() => (showRecent = !showRecent)}>{showRecent ? 'Hide' : 'Show'} finished ({recent.length})</button>{/if}
    </header>
    {#each active as j (j.id)}
      <div class="job {j.status}">
        <div class="row">
          <span>{icon[j.kind]}</span>
          <button class="link job-title" onclick={() => openThread(j)} title={j.title}>{j.title}</button>
          <span class="badge">{j.status === 'queued' ? 'waiting' : 'working'}</span>
          <button class="small danger" onclick={() => stop(j)}>Stop</button>
        </div>
        {#if j.status === 'running'}<JobLog job={j} />{/if}
      </div>
    {/each}
    {#if showRecent}
      {#each recent as j (j.id)}
        <div class="job {j.status}">
          <div class="row">
            <span>{icon[j.kind]}</span>
            <button class="link job-title" onclick={() => (open[j.id] = !open[j.id])} title="Show what the AI did">{j.title}</button>
            <span class="badge st-{j.status}">{j.status}</span>
          </div>
          {#if open[j.id]}<JobLog job={j} lines={14} />{/if}
        </div>
      {/each}
    {/if}
  </section>
{/if}
