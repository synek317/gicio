<script lang="ts">
  import type { SendMode } from '../types';

  let { send, disabled = false, small = false, task = false }: { send: (mode: SendMode) => void; disabled?: boolean; small?: boolean; task?: boolean } = $props();
</script>

{#if task}
<button class:small onclick={() => send('ask')} {disabled} title="The AI answers or does it right now">Ask now</button>
<button class="primary" class:small onclick={() => send('todo')} {disabled} title="Runs in the background (one task at a time); code changes are added as a step at the end of the queue">Start AI task <kbd>⌘↵</kbd></button>
{:else}
<button class:small onclick={() => send('note')} {disabled} title="Save as your own note; the AI is not involved">Note</button>
<button class:small onclick={() => send('review')} {disabled} title="Keep it as a pending review comment. Collect several, then send them all to the AI at once with “Submit review”.">Add to review</button>
<button class:small onclick={() => send('todo')} {disabled} title="Queue it as a task for the AI. It works in the background (one task at a time) and its code changes are added as a step at the end of the queue. You keep reviewing meanwhile.">Add AI to-do</button>
<button class="primary" class:small onclick={() => send('ask')} {disabled} title="The AI answers now, in this thread">Ask now <kbd>⌘↵</kbd></button>
{/if}
