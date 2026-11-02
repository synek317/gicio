<script lang="ts">
  import { tick } from 'svelte';
  import { runSearch, search } from '../project-search.svelte';
  import ExtensionChips from './ExtensionChips.svelte';
  import FindToggles from './FindToggles.svelte';
  import SearchFile from './SearchFile.svelte';

  /** Search in all files of the project (⇧⌘F), with "files to include / exclude" like VS Code. */
  let input: HTMLInputElement | undefined = $state();

  $effect(() => {
    void search.focus;
    tick().then(() => {
      input?.focus();
      input?.select();
    });
  });
  $effect(() => {
    void search.query;
    void search.opts;
    void search.include;
    void search.exclude;
    runSearch();
  });

  const r = $derived(search.results);
  const allCollapsed = $derived(!!r?.files.length && r.files.every((f) => search.collapsed[f.path]));

  function toggleAll() {
    search.collapsed = allCollapsed ? {} : Object.fromEntries((r?.files ?? []).map((f) => [f.path, true]));
  }

  const ALT_TOGGLES: Record<string, keyof typeof search.opts> = { KeyC: 'caseSensitive', KeyW: 'wholeWord', KeyR: 'regex' };

  function keydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      runSearch(0, true);
    } else if (e.altKey && !e.metaKey && !e.ctrlKey && ALT_TOGGLES[e.code]) {
      e.preventDefault();
      const key = ALT_TOGGLES[e.code];
      search.opts = { ...search.opts, [key]: !search.opts[key] };
    }
  }
</script>

<div class="search-panel">
  <div class="search-box">
    <input
      bind:this={input}
      bind:value={search.query}
      class:invalid={!!search.error}
      placeholder="Search in project"
      title="Enter: search again · ⌥C / ⌥W / ⌥R: case, word, regex"
      spellcheck="false"
      autocomplete="off"
      onkeydown={keydown}
    />
    <FindToggles bind:opts={search.opts} />
  </div>
  <button class="link small filters-toggle" onclick={() => (search.showFilters = !search.showFilters)}>
    {search.showFilters ? '▾' : '▸'} Files to include / exclude{#if !search.showFilters && (search.include || search.exclude)} <span class="muted">(set)</span>{/if}
  </button>
  {#if search.showFilters}
    <label class="search-filter">
      <span class="muted small">files to include</span>
      <input bind:value={search.include} placeholder="e.g. *.rs, src/lib" spellcheck="false" autocomplete="off" onkeydown={keydown} />
    </label>
    <label class="search-filter">
      <span class="muted small">files to exclude</span>
      <input bind:value={search.exclude} placeholder="e.g. tests, *.lock" spellcheck="false" autocomplete="off" onkeydown={keydown} />
    </label>
  {/if}

  <div class="search-status small">
    {#if search.busy}<span class="spinner"></span>{/if}
    {#if search.error}
      <span class="error">{search.error}</span>
    {:else if r}
      <span class="muted">
        {r.matches} result{r.matches === 1 ? '' : 's'} in {r.files.length} file{r.files.length === 1 ? '' : 's'}{#if r.truncated}, stopped at the limit{/if}
      </span>
    {/if}
    <span class="spacer"></span>
    {#if r?.files.length}
      <button class="ghost small" onclick={toggleAll} title={allCollapsed ? 'Expand all' : 'Collapse all'}>{allCollapsed ? '⊞' : '⊟'}</button>
    {/if}
    {#if search.query}<button class="ghost small" onclick={() => runSearch(0, true)} title="Search again">⟳</button>{/if}
  </div>
  {#if r}<ExtensionChips files={r.files} />{/if}

  {#each r?.files ?? [] as file (file.path)}<SearchFile {file} />{/each}
</div>
