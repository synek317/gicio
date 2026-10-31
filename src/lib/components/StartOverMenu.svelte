<script lang="ts">
  import { plan, run, startOver } from '../actions';
  import { api } from '../api';
  import { app } from '../state.svelte';
  import ReviewSettingsModal from './ReviewSettingsModal.svelte';

  let open = $state(false);
  let showSettings = $state(false);

  function act(f: () => void) {
    open = false;
    f();
  }

  function otherRepo() {
    app.snap = null;
    app.view = null;
  }
</script>

<div class="menu-anchor split-button">
  <button onclick={startOver} title="Discard all review progress and start again">Start over</button>
  <button class="caret" onclick={() => (open = !open)} title="More: change what and how you review">▾</button>
  {#if open}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="menu-bg" onclick={() => (open = false)}></div>
    <div class="menu dropdown">
      <button onclick={() => act(() => (showSettings = true))}>Change review settings… <span class="muted">base, staging, order, AI</span></button>
      <hr />
      <button onclick={() => act(() => plan(false))}>Re-plan all steps with AI <span class="muted">keeps your marks</span></button>
      <button onclick={() => act(() => run(api.planByFiles()))}>Review file by file <span class="muted">keeps your marks</span></button>
      <hr />
      <button onclick={() => act(startOver)}>Start over <span class="muted">discard marks and discussions</span></button>
      <button onclick={() => act(otherRepo)}>Open another repository…</button>
    </div>
  {/if}
</div>

{#if showSettings}<ReviewSettingsModal close={() => (showSettings = false)} />{/if}
