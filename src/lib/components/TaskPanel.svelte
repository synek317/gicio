<script lang="ts">
  import { run } from '../actions';
  import { api } from '../api';
  import { implementRequirements, needsWork } from '../intent-actions';
  import { renderMarkdown } from '../markdown';
  import { app } from '../state.svelte';
  import type { Verdict } from '../types';
  import IntentField from './IntentField.svelte';
  import RequirementCard from './RequirementCard.svelte';

  /** What the change was supposed to do, and how far it does that. */
  const s = $derived(app.snap!);
  let editing = $state(false);
  let draft = $state('');
  let showDismissed = $state(false);

  const live = $derived(s.requirements.filter((r) => r.status !== 'dismissed'));
  const dismissed = $derived(s.requirements.filter((r) => r.status === 'dismissed'));
  const toDo = $derived(live.filter(needsWork));
  const beyond = $derived(s.findings.filter((f) => f.source === 'intent' && f.status === 'open').length);
  const count = (v: Verdict) => live.filter((r) => r.verdict === v).length;

  function edit() {
    draft = s.intent?.text ?? '';
    editing = true;
  }

  async function save() {
    if (await run(api.setIntent(draft.trim() || null))) editing = false;
  }
</script>

<div class="threads-body task-panel">
  {#if editing || !s.intent}
    <div class="card">
      <label class="task-label" for="task-intent">What should this change do?</label>
      <IntentField id="task-intent" bind:value={draft} />
      {#if s.intentChecked}<p class="warn-text small">Changing the text discards the result of the last check.</p>{/if}
      <div class="row">
        <button class="primary small" disabled={!draft.trim() && !s.intent} onclick={save}>Save</button>
        {#if editing}<button class="ghost small" onclick={() => (editing = false)}>Cancel</button>{/if}
      </div>
    </div>
  {:else}
    <div class="card">
      <div class="row">
        <strong class="task-label">The task</strong>
        {#if s.intent.source === 'commits'}<span class="badge" title="Taken from the commit messages of the change, which whoever made the change wrote. Replace it with the real task if you have it.">from commit messages</span>{/if}
        <span class="spacer"></span>
        <button class="ghost small" onclick={edit}>Edit</button>
      </div>
      <div class="md task-text">{@html renderMarkdown(s.intent.text)}</div>
    </div>
    <div class="row wrap">
      <button class:primary={!s.intentChecked || s.intentStale} class="small" disabled={s.checkingIntent} onclick={() => run(api.checkIntent())}>{s.intentChecked ? 'Check again' : 'Check the change against the task'}</button>
      {#if s.checkingIntent}<span class="spinner"></span>{/if}
    </div>
  {/if}

  {#if s.intentError}<div class="error small">{s.intentError}</div>{/if}
  {#if s.intentStale}<p class="warn-text small">The code changed after this check, so the results below may be out of date.</p>{/if}

  {#if s.intentChecked && !editing}
    {#if s.requirements.length}
      <div class="row wrap task-counts muted small">
        <span>{count('done')} done</span>
        {#if count('partial')}<span>· {count('partial')} partly</span>{/if}
        {#if count('missing')}<span>· {count('missing')} missing</span>{/if}
        {#if count('unclear')}<span>· {count('unclear')} cannot tell</span>{/if}
        <span class="spacer"></span>
        {#if toDo.length > 1}<button class="small primary" onclick={() => implementRequirements(toDo)}>Implement all ({toDo.length})</button>{/if}
      </div>
    {:else}
      <p class="muted small hint">The AI found nothing checkable in this description. A more specific one gives better results.</p>
    {/if}
    {#each live as r (r.id)}<RequirementCard requirement={r} />{/each}
    {#if beyond}
      <button class="link small" onclick={() => (app.rightTab = 'findings')}>{beyond} change{beyond === 1 ? '' : 's'} the task did not ask for, in Findings</button>
    {/if}
    {#if dismissed.length}
      <button class="qhead toggle-head" onclick={() => (showDismissed = !showDismissed)}>{showDismissed ? '▾' : '▸'} Dismissed <span class="muted">({dismissed.length})</span></button>
      {#if showDismissed}
        {#each dismissed as r (r.id)}<RequirementCard requirement={r} />{/each}
      {/if}
    {/if}
  {:else if !s.intent && !editing}
    <p class="muted small hint">With a description of the task, the AI plans and reviews with the purpose of the change in mind, and it can check what is missing and what nobody asked for.</p>
  {/if}
</div>
