<script lang="ts">
  import { plan, run, startOver } from '../actions';
  import { api } from '../api';
  import { leaveReview } from '../nav';
  import { app } from '../state.svelte';
  import ReviewSettingsModal from './ReviewSettingsModal.svelte';

  /** Everything about the review as a whole: its settings, starting over, leaving. How it is split up
   *  (AI or file by file, how many steps) lives in the review settings. */
  let open = $state(false);
  let showSettings = $state(false);
  const s = $derived(app.snap!);

  function act(f: () => void) {
    open = false;
    f();
  }
</script>

<div class="menu-anchor">
  <button onclick={() => (open = !open)}>{#if s.planning}<span class="spinner"></span> Planning{:else}Review{/if} ▾</button>
  {#if open}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="menu-bg" onclick={() => (open = false)}></div>
    <div class="menu dropdown">
      {#if s.plannedByAi}
        <button disabled={s.planning} onclick={() => act(() => plan(true, s.stepCount))}>Re-plan remaining steps <span class="muted">steps you have not started</span></button>
        <hr />
      {/if}
      <button onclick={() => act(() => (showSettings = true))}>Review settings… <span class="muted">base, staging, order, AI</span></button>
      <button onclick={() => act(() => run(api.refresh()))}>Reload from disk <kbd>r</kbd></button>
      <hr />
      <button onclick={() => act(() => (app.showFinish = true))}>Finish review… <span class="muted">stage, commit or amend, then close</span></button>
      <hr />
      <button onclick={() => act(startOver)}>Start over… <span class="muted">discard marks and conversations, back to the start screen</span></button>
      <button onclick={() => act(leaveReview)}>Open another repository…</button>
    </div>
  {/if}
</div>

{#if showSettings}<ReviewSettingsModal close={() => (showSettings = false)} />{/if}
