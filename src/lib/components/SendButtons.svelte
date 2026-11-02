<script lang="ts">
  import type { SendMode } from '../types';

  /** The two things a message can do: start (or continue) a conversation now, or wait for later. */
  let { send, disabled = false, small = false, reply = false }: { send: (mode: SendMode) => void; disabled?: boolean; small?: boolean; reply?: boolean } = $props();
</script>

<button class:small onclick={() => send('later')} {disabled} title="Leave it for later. The “Resolve issues left for later” step hands everything left for later to the AI at once; you can also resolve items yourself.">{reply ? 'Later' : 'Add for later'}</button>
<button class="primary" class:small onclick={() => send('ask')} {disabled} title="The AI answers now, in this conversation">{reply ? 'Send' : 'Start conversation'} <kbd>⌘↵</kbd></button>
