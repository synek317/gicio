<script lang="ts">
  import type { FindOptions } from '../find-query';

  /** Match case / whole word / regular expression switches, as in VS Code. */
  let { opts = $bindable() }: { opts: FindOptions } = $props();

  const TOGGLES = [
    ['caseSensitive', 'Aa', 'Match case'],
    ['wholeWord', 'ab', 'Match whole word'],
    ['regex', '.*', 'Use regular expression'],
  ] as const;
</script>

<span class="find-toggles">
  {#each TOGGLES as [key, label, title]}
    <button
      class="find-toggle"
      class:on={opts[key]}
      class:word={key === 'wholeWord'}
      aria-pressed={opts[key]}
      {title}
      onclick={() => (opts = { ...opts, [key]: !opts[key] })}>{label}</button
    >
  {/each}
</span>
