<script lang="ts">
  import { go } from '../actions';
  import { app } from '../state.svelte';

  /** The last step: everything put off, lines and conversations, to resolve at the end. */
  let { label }: { label: string } = $props();

  const active = $derived(app.view?.kind === 'later');
  const lines = $derived(app.overall.deferred);
  const issues = $derived(app.laterThreads.length);
  const done = $derived(!lines && !issues);
</script>

<li class="qrow">
  <button class="qitem later-step" class:done class:deferred={!done} class:active onclick={() => go({ kind: 'later' })}>
    <span class="qicon">{done ? '✓' : '⏸'}</span>
    <span class="qtext">
      <span class="qtitle">{label}. Resolve issues left for later</span>
      {#if !done}<span class="qmeta">{#if issues}{issues} conversation(s){/if}{#if issues && lines} · {/if}{#if lines}{lines} line(s){/if}</span>{/if}
    </span>
    {#if done}<span class="stamp">Done</span>{/if}
  </button>
</li>
