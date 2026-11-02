<script lang="ts">
  /** A text field with suggestions: pick one, or type anything. Suggestions filter as you type. */
  let { value = $bindable(), options, placeholder = '' }: { value: string; options: string[]; placeholder?: string } = $props();

  let open = $state(false);
  let active = $state(-1);
  let input: HTMLInputElement | undefined = $state();
  // Fixed under the field, so a dialog's scrolling body does not clip the list.
  let place = $state('');
  $effect(() => {
    if (!open || !input) return;
    const r = input.getBoundingClientRect();
    place = `left:${r.left}px; top:${r.bottom + 2}px; width:${r.width}px; max-height:${Math.max(120, Math.min(260, window.innerHeight - r.bottom - 12))}px`;
  });
  const shown = $derived.by(() => {
    const q = value.trim().toLowerCase();
    // A value picked from the list shows the whole list again, so it is easy to switch.
    if (!q || options.some((o) => o.toLowerCase() === q)) return options;
    return options.filter((o) => o.toLowerCase().includes(q));
  });

  function pick(o: string) {
    value = o;
    open = false;
  }

  function key(e: KeyboardEvent) {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      open = true;
      const n = shown.length;
      if (n) active = e.key === 'ArrowDown' ? (active + 1) % n : active <= 0 ? n - 1 : active - 1;
    } else if (e.key === 'Enter' && open && shown[active]) {
      e.preventDefault();
      pick(shown[active]);
    } else if (e.key === 'Escape' && open) {
      e.stopPropagation();
      open = false;
    }
  }
</script>

<div class="combo">
  <input
    bind:this={input}
    bind:value
    {placeholder}
    spellcheck="false"
    autocomplete="off"
    onfocus={() => ((open = true), (active = -1))}
    onblur={() => (open = false)}
    oninput={() => ((open = true), (active = -1))}
    onkeydown={key}
  />
  {#if open && shown.length}
    <ul class="combo-list" role="listbox" style={place}>
      {#each shown as o, i}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <li role="option" aria-selected={i === active} class:active={i === active} class:current={o === value} onmousedown={(e) => (e.preventDefault(), pick(o))}>{o}</li>
      {/each}
    </ul>
  {/if}
</div>
