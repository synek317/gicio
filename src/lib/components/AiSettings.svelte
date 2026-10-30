<script lang="ts">
  import { run } from '../actions';
  import { api } from '../api';
  import { fromBackend, toBackend, type BackendForm } from '../backend-form';
  import { app } from '../state.svelte';
  import BackendPicker from './BackendPicker.svelte';

  let { close }: { close: () => void } = $props();
  let form = $state<BackendForm>(fromBackend(app.snap!.backend));

  async function apply() {
    if (await run(api.setBackend(toBackend(form)))) close();
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && close()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="modal-bg" onclick={close}>
  <div class="modal ai-settings" onclick={(e) => e.stopPropagation()}>
    <h2>AI for this review</h2>
    <p class="muted small">Used for the next questions, to-dos and planning. Running jobs keep their settings.</p>
    <BackendPicker bind:form />
    <h3>Project instructions</h3>
    {#if app.snap?.instructions.length}
      <ul class="instr">
        {#each app.snap.instructions as f}
          <li><code>{f.path}</code>{#if f.skill} <span class="muted">skill “{f.skill[0]}”</span>{/if} <span class="badge">{f.native ? 'read by the CLI itself' : 'added to prompts by gicio'}</span></li>
        {/each}
      </ul>
    {:else}
      <p class="muted small">None found (CLAUDE.md, AGENTS.md, .github/copilot-instructions.md, .cursor/rules, skills…).</p>
    {/if}
    <div class="row"><span class="spacer"></span><button onclick={close}>Cancel</button><button class="primary" onclick={apply}>Apply</button></div>
  </div>
</div>
