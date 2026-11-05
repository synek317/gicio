<script lang="ts">
  import type { BackendForm } from '../backend-form';
  import { api } from '../api';
  import { languageOptions } from '../languages';
  import type { BackendKind, Effort } from '../types';
  import ComboBox from './ComboBox.svelte';

  let { form = $bindable() }: { form: BackendForm } = $props();

  /** Known models, for when the CLI says nothing; whatever the installed CLI names comes first. */
  const MODELS: Record<BackendKind, string[]> = {
    claude: ['fable', 'opus', 'sonnet', 'haiku'],
    codex: ['gpt-6.1-sol', 'gpt-6-astra', 'gpt-6.1-terra', 'gpt-6-sol', 'gpt-6-luna', 'gpt-5.6-sol', 'gpt-5.6-terra', 'gpt-5.6-luna'],
    copilot: ['claude-sonnet-4.5', 'gpt-5.6-sol', 'gpt-5.6-terra', 'gpt-5.6-luna'],
    custom: [],
  };
  let detected = $state<Partial<Record<BackendKind, string[]>>>({});
  $effect(() => {
    const kind = form.kind;
    if (kind === 'custom' || detected[kind]) return;
    api.aiModels(kind).then((m) => (detected = { ...detected, [kind]: m })).catch(() => {});
  });
  // Codex gives the full list for the account, so models it dropped must not linger; the help of the
  // other CLIs only names examples.
  const models = $derived(form.kind === 'codex' && detected.codex?.length ? detected.codex : [...new Set([...(detected[form.kind] ?? []), ...MODELS[form.kind]])]);
  const languages = languageOptions();
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
    <label>Model <ComboBox bind:value={form.model} options={models} placeholder="CLI default" /></label>
  {/if}
  <label title={form.kind === 'custom' ? 'Not supported for custom commands' : 'Claude: --effort. Copilot: --reasoning-effort. Codex: model_reasoning_effort (xhigh/max = high).'}>Reasoning
    <select bind:value={form.effort} disabled={form.kind === 'custom'}>
      <option value="">CLI default</option>
      {#each EFFORTS as e}<option value={e}>{e}</option>{/each}
    </select>
  </label>
  <label>AI writes in <ComboBox bind:value={form.language} options={languages} /></label>
</div>
{#if missing}
  <div class="tool-missing">
    <strong>{form.kind}</strong> is not installed. gicio runs the AI through its command-line tool, signed in with your own account. Install it with
    <code>{missing.cmd}</code>, {missing.login}. If it is installed somewhere unusual, set <code>GICIO_{form.kind.toUpperCase()}_BIN</code>.
  </div>
{/if}
