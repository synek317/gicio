<script lang="ts">
  import { revealThread } from '../actions';
  import { api } from '../api';
  import { app } from '../state.svelte';
  import type { JobInfo, JobKind } from '../types';
  import JobLog from './JobLog.svelte';

  // A small indicator in the status bar; the job list opens on demand instead of taking room
  // in the discussions.
  const active = $derived(app.jobs.filter((j) => j.status === 'running' || j.status === 'queued'));
  const recent = $derived(app.jobs.filter((j) => j.status !== 'running' && j.status !== 'queued').slice(-8).reverse());
  let show = $state(false);
  let open = $state<Record<string, boolean>>({});
  const icon: Record<JobKind, string> = { ask: '💬', plan: '🗺', chat: '🗨', summary: '📝', review: '✉️', aiReview: '🔎', intentCheck: '🎯', fix: '🛠', commit: '✔', tool: '⚙' };

  function stop(j: JobInfo) {
    api.cancelJob(j.id).catch((e) => app.toast(String(e), 'error'));
  }

  function openJob(j: JobInfo) {
    const t = app.snap?.threads.find((t) => t.id === j.thread);
    if (t) {
      revealThread(t);
      show = false;
    } else open[j.id] = !open[j.id];
  }
</script>

{#if active.length || recent.length}
  <span class="menu-anchor">
    <button class="ghost small" class:busy={active.length} onclick={() => (show = !show)} title="What the AI is doing and has done">
      {#if active.length}<span class="spinner"></span> AI: {active.length} working{:else}AI jobs{/if}
    </button>
    {#if show}
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
      <div class="pop-bg" onclick={() => (show = false)}></div>
      <section class="activity-pop">
        {#each active as j (j.id)}
          <div class="job">
            <div class="row">
              <span>{icon[j.kind]}</span>
              <button class="link job-title" onclick={() => openJob(j)} title={j.title}>{j.title}</button>
              <span class="badge">{j.status === 'queued' ? 'waiting' : 'working'}</span>
              <button class="small danger" onclick={() => stop(j)}>Stop</button>
            </div>
            {#if j.status === 'running'}<JobLog job={j} compact />{/if}
          </div>
        {/each}
        {#if recent.length}
          <h4 class="muted">Finished</h4>
          {#each recent as j (j.id)}
            <div class="job">
              <div class="row">
                <span>{icon[j.kind]}</span>
                <button class="link job-title" onclick={() => openJob(j)} title={j.title}>{j.title}</button>
                <button class="ghost small" onclick={() => (open[j.id] = !open[j.id])} title="Show what the AI did">log</button>
                <span class="badge st-{j.status}">{j.status}</span>
              </div>
              {#if open[j.id]}<JobLog job={j} lines={14} />{/if}
            </div>
          {/each}
        {/if}
      </section>
    {/if}
  </span>
{/if}
