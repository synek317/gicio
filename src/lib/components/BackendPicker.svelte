<script lang="ts">
  import type { BackendForm } from '../backend-form';
  import { api } from '../api';
  import type { BackendKind, Effort } from '../types';

  let { form = $bindable() }: { form: BackendForm } = $props();

  const MODELS: Record<BackendKind, string[]> = {
    claude: ['fable', 'opus', 'sonnet', 'haiku'],
    codex: ['gpt-5-codex', 'gpt-5'],
    copilot: ['claude-sonnet-4.5', 'gpt-5'],
    custom: [],
  };
  const EFFORTS: Effort[] = ['low', 'medium', 'high', 'xhigh', 'max'];

  /** How to get each CLI; they sign in with your own account (Codex: your ChatGPT plan). */
  const INSTALL: Record<string, { cmd: string; login: string }> = {
    claude: { cmd: 'npm install -g @anthropic-ai/claude-code', login: 'then run `claude` once and log in' },
    codex: { cmd: 'npm install -g @openai/codex', login: 'then `codex login` and choose “Sign in with ChatGPT” (or install the ChatGPT desktop app: gicio uses the Codex inside it automatically)' },
    copilot: { cmd: 'npm install -g @github/copilot', login: 'then run `copilot` once and log in with GitHub (or install the GitHub Copilot desktop app: gicio uses the CLI it downloads)' },
  };
  let installed = $state<Record<string, boolean> | null>(null);
  api.aiTools().then((t) => (installed = t)).catch(() => {});
  const missing = $derived(installed && form.kind !== 'custom' && installed[form.kind] === false ? INSTALL[form.kind] : null);
  const tag = (k: string) => (installed && installed[k] === false ? ' — not installed' : '');
</script>

<div class="grid2">
  <label>AI
    <select bind:value={form.kind}>
      <option value="claude">Claude Code (claude -p){tag('claude')}</option>
      <option value="codex">OpenAI Codex / ChatGPT (codex exec){tag('codex')}</option>
      <option value="copilot">GitHub Copilot CLI (copilot -p){tag('copilot')}</option>
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
  <label title={form.kind === 'custom' ? 'Not supported for custom commands' : 'Claude: --effort. Copilot: --reasoning-effort. Codex: model_reasoning_effort (xhigh/max = high).'}>Reasoning
    <select bind:value={form.effort} disabled={form.kind === 'custom'}>
      <option value="">CLI default</option>
      {#each EFFORTS as e}<option value={e}>{e}</option>{/each}
    </select>
  </label>
  <label>AI writes in <input bind:value={form.language} spellcheck="false" /></label>
</div>
{#if missing}
  <div class="tool-missing">
    <strong>{form.kind}</strong> is not installed. gicio runs the AI through its command-line tool, signed in with your own account. Install it with
    <code>{missing.cmd}</code>, {missing.login}. If it is installed somewhere unusual, set <code>GICIO_{form.kind.toUpperCase()}_BIN</code>.
  </div>
{/if}
