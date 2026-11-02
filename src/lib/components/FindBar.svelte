<script lang="ts">
  import { tick } from 'svelte';
  import { closeFind, find, findStep, refreshFind } from '../find.svelte';
  import FindToggles from './FindToggles.svelte';

  /** Find in what the main view shows (⌘F): the diff of the step, or the open file. */
  let input: HTMLInputElement | undefined = $state();

  $effect(() => {
    void find.focus;
    tick().then(() => {
      input?.focus();
      input?.select();
    });
  });
  $effect(() => {
    void find.query;
    void find.opts;
    refreshFind();
  });

  const label = $derived(find.error ? 'Invalid pattern' : !find.query ? '' : find.count ? `${find.current < 0 ? '?' : find.current + 1} of ${find.count}` : 'No results');

  const ALT_TOGGLES: Record<string, keyof typeof find.opts> = { KeyC: 'caseSensitive', KeyW: 'wholeWord', KeyR: 'regex' };

  function keydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      findStep(e.shiftKey ? -1 : 1);
    } else if (e.key === 'Escape') {
      e.stopPropagation();
      closeFind();
    } else if (e.altKey && !e.metaKey && !e.ctrlKey && ALT_TOGGLES[e.code]) {
      e.preventDefault();
      const key = ALT_TOGGLES[e.code];
      find.opts = { ...find.opts, [key]: !find.opts[key] };
    }
  }
</script>

<div class="find-bar" role="search">
  <input
    bind:this={input}
    bind:value={find.query}
    class:invalid={!!find.error}
    title={find.error || 'Enter: next match · ⇧Enter: previous · ⌥C / ⌥W / ⌥R: case, word, regex'}
    placeholder="Find"
    spellcheck="false"
    autocomplete="off"
    onkeydown={keydown}
  />
  <FindToggles bind:opts={find.opts} />
  <span class="find-count" class:none={!!find.query && !find.count}>{label}</span>
  <button class="ghost" disabled={!find.count} onclick={() => findStep(-1)} title="Previous match (⇧Enter)" aria-label="Previous match">↑</button>
  <button class="ghost" disabled={!find.count} onclick={() => findStep(1)} title="Next match (Enter)" aria-label="Next match">↓</button>
  <button class="ghost" onclick={closeFind} title="Close (Esc)" aria-label="Close">✕</button>
</div>
