<script lang="ts">
  import { plan, run } from '../actions';
  import { api } from '../api';
  import { settings } from '../app-settings.svelte';
  import { fromBackend, toBackend, type BackendForm } from '../backend-form';
  import { openReview } from '../open-review';
  import { baseOf, optionsOf, stageOf, type StartOptions } from '../start-options';
  import { app } from '../state.svelte';
  import BackendPicker from './BackendPicker.svelte';
  import ReviewOptions from './ReviewOptions.svelte';

  let { close }: { close: () => void } = $props();
  const snap = app.snap!;
  const initial = optionsOf(snap);
  let options = $state<StartOptions>({ ...initial });
  let form = $state<BackendForm>(fromBackend(snap.backend));
  let ignoreWhitespace = $state(snap.diffOptions.ignoreWhitespace);
  let asDefault = $state(false);
  let busy = $state(false);

  /** Applies only what changed: another base reopens the review, the rest is switched in place. */
  async function apply() {
    busy = true;
    try {
      if (asDefault) await settings.save({ ...settings.value, start: $state.snapshot(options), defaultBackend: $state.snapshot(form) });
      if (baseOf(options) !== baseOf(initial)) {
        await openReview(snap.repo, options, form);
      } else {
        if (JSON.stringify(toBackend(form)) !== JSON.stringify(snap.backend)) await run(api.setBackend(toBackend(form)));
        if (stageOf(options) !== snap.stageMode) await run(api.setStageMode(stageOf(options)));
        if (ignoreWhitespace !== snap.diffOptions.ignoreWhitespace) await run(api.setDiffOptions({ ignoreWhitespace }));
        if (options.order !== initial.order) {
          if (options.order === 'ai') plan(false);
          else await run(api.planByFiles());
        }
      }
      close();
    } catch (e) {
      app.toast(String(e), 'error');
    } finally {
      busy = false;
    }
  }
</script>

<svelte:window onkeydown={(e) => e.key === 'Escape' && close()} />

<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
<div class="modal-bg" onclick={close}>
  <div class="modal settings" onclick={(e) => e.stopPropagation()}>
    <h2>Review settings</h2>
    <p class="muted small"><code>{snap.repo}</code></p>
    <div class="settings-body">
      <ReviewOptions bind:options repo={snap.repo} idPrefix="review" />
      <label class="toggle"><input type="checkbox" bind:checked={ignoreWhitespace} /> Ignore whitespace-only changes</label>
      <BackendPicker bind:form />
      {#if baseOf(options) !== baseOf(initial)}
        <p class="warn-text small">A different base opens a separate review (your progress on this one is kept and comes back when you switch back).</p>
      {/if}
      {#if stageOf(options) === 'index' && snap.stageMode !== 'index'}
        <p class="warn-text small">Everything currently staged will be unstaged, then accepted lines are staged.</p>
      {/if}
    </div>
    <footer class="row">
      <label class="toggle"><input type="checkbox" bind:checked={asDefault} /> Also use as defaults for new reviews</label>
      <span class="spacer"></span>
      <button onclick={close}>Cancel</button>
      <button class="primary" disabled={busy || (options.scope === 'since' && !options.base)} onclick={apply}>Apply</button>
    </footer>
  </div>
</div>
