<script lang="ts">
  import { exportReview, reviewWithAi, runTool } from '../review-actions';
  import { run } from '../actions';
  import { api } from '../api';
  import { app } from '../state.svelte';

  let open = $state(false);
  const s = $derived(app.snap!);
  const rust = $derived(s.files.some((f) => f.path.endsWith('.rs') || f.path === 'Cargo.toml'));
  const chunkId = $derived(app.view?.kind === 'chunk' ? app.view.id : null);

  function act(f: () => void) {
    open = false;
    f();
  }
</script>

<div class="menu-anchor">
  <button onclick={() => (open = !open)}>Tools ▾</button>
  {#if open}
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
    <div class="menu-bg" onclick={() => (open = false)}></div>
    <div class="menu dropdown">
      <button onclick={() => act(() => reviewWithAi({ kind: 'all' }))}>AI review: whole change</button>
      <button disabled={!chunkId} onclick={() => act(() => chunkId && reviewWithAi({ kind: 'chunk', id: chunkId }))}>AI review: this step</button>
      <hr />
      <button disabled={!rust} onclick={() => act(() => runTool('fmt'))}>cargo fmt</button>
      <button disabled={!rust} onclick={() => act(() => runTool('clippy'))}>cargo clippy → findings</button>
      <button disabled={!rust} onclick={() => act(() => runTool('clippyFix'))}>cargo clippy --fix</button>
      <hr />
      <button onclick={() => act(() => run(api.setDiffOptions({ ignoreWhitespace: !s.diffOptions.ignoreWhitespace })))}>
        {s.diffOptions.ignoreWhitespace ? '✓ ' : ''}Ignore whitespace changes
      </button>
      <button onclick={() => act(exportReview)}>Export review as markdown…</button>
    </div>
  {/if}
</div>
