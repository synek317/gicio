<script lang="ts">
  import { api } from '../api';
  import type { StartOptions } from '../start-options';
  import StepCountPicker from './StepCountPicker.svelte';

  let { options = $bindable(), repo = null, idPrefix = 'opt' }: { options: StartOptions; repo?: string | null; idPrefix?: string } = $props();
  let refs = $state<string[]>([]);

  $effect(() => {
    const r = repo;
    if (!r) return;
    api.listRefs(r).then((x) => (refs = x)).catch(() => (refs = []));
  });
</script>

<fieldset>
  <legend>What to review</legend>
  <label class="choice"><input type="radio" bind:group={options.scope} value="uncommitted" />
    <span><strong>Uncommitted changes</strong><br /><span class="muted small">Staged, unstaged and new files, compared to HEAD.</span></span>
  </label>
  <label class="choice"><input type="radio" bind:group={options.scope} value="since" />
    <span><strong>Everything since a branch or commit</strong><br /><span class="muted small">Commits on top of it plus uncommitted changes, e.g. an AI's branch against <code>main</code>.</span></span>
  </label>
  {#if options.scope === 'since'}
    <div class="indent">
      <input bind:value={options.base} list="{idPrefix}-refs" spellcheck="false" placeholder="main" />
      <datalist id="{idPrefix}-refs">{#each refs as r}<option value={r}></option>{/each}</datalist>
      <div class="chips">{#each ['main', 'master', 'origin/main', 'HEAD~1'] as r}<button class="chip" onclick={() => (options.base = r)}>{r}</button>{/each}</div>
    </div>
  {/if}
</fieldset>

<fieldset>
  <legend>Accepted changes</legend>
  <label class="choice"><input type="radio" bind:group={options.stage} value="memory" />
    <span><strong>Remember them in gicio only</strong><br /><span class="muted small">Your git index is not touched.</span></span>
  </label>
  <label class="choice" class:disabled={options.scope !== 'uncommitted'}><input type="radio" bind:group={options.stage} value="index" disabled={options.scope !== 'uncommitted'} />
    <span><strong>Stage them in git</strong><br /><span class="muted small">
      {#if options.scope !== 'uncommitted'}Only for uncommitted changes.{:else}Accepted lines are staged (also parts of files), everything else is not, so you can commit exactly what you reviewed.
      <span class="warn-text">Everything currently staged is unstaged first.</span>{/if}
    </span></span>
  </label>
</fieldset>

<fieldset>
  <legend>Order of review</legend>
  <label class="choice"><input type="radio" bind:group={options.order} value="ai" />
    <span><strong>Logical steps planned by AI</strong><br /><span class="muted small">Groups related changes (also parts of files) and orders them so each step builds on the previous one. Takes a minute; you can start file by file meanwhile.</span></span>
  </label>
  {#if options.order === 'ai'}
    <div class="indent">
      <span class="muted small">Number of steps</span>
      <StepCountPicker bind:value={options.steps} />
    </div>
  {/if}
  <label class="choice"><input type="radio" bind:group={options.order} value="files" />
    <span><strong>File by file</strong><br /><span class="muted small">No planning.</span></span>
  </label>
</fieldset>
