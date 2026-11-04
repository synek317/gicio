<script lang="ts">
  import { DEFAULT_STEP_TOTAL, MAX_STEP_TOTAL, parseStepTotal, STEP_PRESETS } from '../step-count';
  import type { StepCount } from '../types';

  /** How many review steps the AI should plan: a general preference or an exact number. */
  let { value = $bindable() }: { value: StepCount } = $props();
  let lastTotal = $state(DEFAULT_STEP_TOTAL);
  const shownTotal = $derived(value.kind === 'exact' ? value.steps : lastTotal);

  function typed(raw: string) {
    const steps = parseStepTotal(raw);
    if (steps === null) return;
    lastTotal = steps;
    value = { kind: 'exact', steps };
  }
</script>

<div class="step-count" role="group" aria-label="Number of steps">
  <div class="seg">
    {#each STEP_PRESETS as p (p.kind)}
      <button class:active={value.kind === p.kind} title={p.hint} onclick={() => (value = { kind: p.kind })}>{p.label}</button>
    {/each}
    <button class:active={value.kind === 'exact'} title="The AI plans exactly this many steps" onclick={() => (value = { kind: 'exact', steps: shownTotal })}>Exactly</button>
  </div>
  {#if value.kind === 'exact'}
    <label class="step-total">
      <input type="number" min="1" max={MAX_STEP_TOTAL} step="1" value={shownTotal} aria-label="Exact number of steps" oninput={(e) => typed(e.currentTarget.value)} onblur={(e) => (e.currentTarget.value = String(shownTotal))} />
      steps
    </label>
  {/if}
</div>
