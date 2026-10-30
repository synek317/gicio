<script lang="ts">
  import type { BackendForm } from '../backend-form';
  import type { BackendKind, Effort } from '../types';

  let { form = $bindable() }: { form: BackendForm } = $props();

  const MODELS: Record<BackendKind, string[]> = {
    claude: ['fable', 'opus', 'sonnet', 'haiku'],
    codex: ['gpt-5-codex', 'gpt-5'],
    copilot: ['claude-sonnet-4.5', 'gpt-5'],
    custom: [],
  };
  const EFFORTS: Effort[] = ['low', 'medium', 'high', 'xhigh', 'max'];
</script>

<div class="grid2">
  <label>AI
    <select bind:value={form.kind}>
      <option value="claude">Claude Code (claude -p)</option>
      <option value="codex">OpenAI Codex (codex exec)</option>
      <option value="copilot">GitHub Copilot CLI (copilot -p)</option>
      <option value="custom">Custom command (prompt on stdin)</option>
    </select>
  </label>
  {#if form.kind === 'custom'}
    <label>Command <input bind:value={form.command} placeholder="ollama run qwen3-coder" spellcheck="false" /></label>
  {:else}
    <label>Model
      <input bind:value={form.model} list="models-{form.kind}" placeholder="CLI default" spellcheck="false" />
      <datalist id="models-{form.kind}">{#each MODELS[form.kind] as m}<option value={m}></option>{/each}</datalist>
    </label>
  {/if}
  <label title={form.kind === 'copilot' || form.kind === 'custom' ? 'Not supported by this CLI; ignored' : 'Claude: --effort. Codex: model_reasoning_effort (xhigh/max = high).'}>Reasoning
    <select bind:value={form.effort} disabled={form.kind === 'copilot' || form.kind === 'custom'}>
      <option value="">CLI default</option>
      {#each EFFORTS as e}<option value={e}>{e}</option>{/each}
    </select>
  </label>
  <label>AI writes in <input bind:value={form.language} spellcheck="false" /></label>
</div>
