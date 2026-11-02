<script lang="ts">
  import { touchesCSharp, touchesRust } from '../project-tools';
  import { exportReview, runTool } from '../review-actions';
  import { app } from '../state.svelte';

  let open = $state(false);
  const rust = $derived(touchesRust(app.snap!.files));
  const csharp = $derived(touchesCSharp(app.snap!.files));

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
      <button disabled={!rust} onclick={() => act(() => runTool('fmt'))}>cargo fmt</button>
      <button disabled={!rust} onclick={() => act(() => runTool('clippy'))}>cargo clippy → findings</button>
      <button disabled={!rust} onclick={() => act(() => runTool('clippyFix'))}>cargo clippy --fix</button>
      <hr />
      <button disabled={!csharp} onclick={() => act(() => runTool('dotnetFormat'))}>dotnet format</button>
      <button disabled={!csharp} onclick={() => act(() => runTool('dotnetBuild'))}>dotnet build → findings</button>
      <hr />
      <button onclick={() => act(exportReview)}>Export review as markdown…</button>
    </div>
  {/if}
</div>
