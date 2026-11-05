<script lang="ts">
  import { MAX_STEP_WISH, STEP_PRESETS } from '../step-count';
  import type { StepCount } from '../types';

  /** How many review steps the AI should plan: a general preference or the reviewer's own words. */
  let { value = $bindable() }: { value: StepCount } = $props();
  /** What was typed, so it comes back after trying a preset. */
  let lastWish = $state('');

  function typed(wish: string) {
    lastWish = wish;
    value = { kind: 'custom', wish };
  }
</script>

<div class="step-count" role="group" aria-label="Number of steps">
  <div class="seg">
    {#each STEP_PRESETS as p (p.kind)}
      <button class:active={value.kind === p.kind} title={p.hint} onclick={() => (value = { kind: p.kind })}>{p.label}</button>
    {/each}
    <button class:active={value.kind === 'custom'} title="Say in your own words what you want" onclick={() => (value = { kind: 'custom', wish: value.kind === 'custom' ? value.wish : lastWish })}>Custom</button>
  </div>
  {#if value.kind === 'custom'}
    <input type="text" maxlength={MAX_STEP_WISH} spellcheck="false" placeholder="e.g. “10 steps” or “as few as possible”" aria-label="Number or size of steps in your own words" value={value.wish} oninput={(e) => typed(e.currentTarget.value)} />
  {/if}
</div>
