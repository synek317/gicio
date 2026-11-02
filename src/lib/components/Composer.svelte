<script lang="ts">
  import type { SendMode } from '../types';
  import EditHint from './EditHint.svelte';
  import SendButtons from './SendButtons.svelte';

  /** Text box of every conversation. `reply` labels the buttons for an ongoing conversation; `oneLine`
   *  starts as one line and shows the options once typed into. Bumping `focus` puts the cursor in it. */
  let {
    send,
    allowEdits = $bindable(true),
    initial = '',
    placeholder,
    reply = false,
    oneLine = false,
    autofocus = false,
    focus = 0,
  }: {
    send: (mode: SendMode, text: string, allowEdits: boolean) => Promise<unknown> | void;
    allowEdits?: boolean;
    initial?: string;
    placeholder: string;
    reply?: boolean;
    oneLine?: boolean;
    autofocus?: boolean;
    focus?: number;
  } = $props();

  // svelte-ignore state_referenced_locally
  let body = $state(initial);
  let sending = $state(false);
  let input: HTMLTextAreaElement | undefined = $state();
  const expanded = $derived(!oneLine || !!body.trim());

  $effect(() => {
    void focus;
    if (autofocus || focus) input?.focus();
  });

  async function submit(mode: SendMode) {
    if (!body.trim() || sending) return;
    sending = true;
    const text = body;
    body = '';
    try {
      await send(mode, text, allowEdits);
    } finally {
      sending = false;
    }
  }

  function key(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey)) {
      e.preventDefault();
      e.stopPropagation();
      submit('ask');
    }
  }
</script>

<textarea class:reply={oneLine} bind:this={input} bind:value={body} onkeydown={key} rows={expanded ? 3 : 1} {placeholder}></textarea>
{#if expanded}
  <div class="send-row"><EditHint bind:checked={allowEdits} /><span class="spacer"></span><SendButtons send={submit} disabled={sending} small={reply} {reply} /></div>
{/if}
