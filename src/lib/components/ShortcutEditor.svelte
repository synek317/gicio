<script lang="ts">
  import { settings } from '../app-settings.svelte';
  import { ACTIONS, comboOf, defaultKeybindings, prettyCombo, type ActionId, type Keybindings } from '../keymap';

  let { keys = $bindable() }: { keys: Keybindings } = $props();

  let waiting = $state<{ id: ActionId; slot: number } | null>(null);

  const owners = $derived.by(() => {
    const m = new Map<string, ActionId[]>();
    for (const a of ACTIONS) for (const k of keys[a.id] ?? []) m.set(k, [...(m.get(k) ?? []), a.id]);
    return m;
  });

  function capture(id: ActionId, slot: number) {
    waiting = { id, slot };
    settings.capturing = true;
  }

  function onKey(e: KeyboardEvent) {
    if (!waiting) return;
    e.preventDefault();
    e.stopPropagation();
    if (e.key === 'Escape') return done();
    const c = comboOf(e);
    if (!c) return;
    const list = [...(keys[waiting.id] ?? [])];
    list[waiting.slot] = c;
    keys = { ...keys, [waiting.id]: list };
    done();
  }

  function done() {
    waiting = null;
    settings.capturing = false;
  }

  function remove(id: ActionId, slot: number) {
    keys = { ...keys, [id]: keys[id].filter((_, i) => i !== slot) };
  }
</script>

<svelte:window onkeydowncapture={onKey} />

<div class="shortcuts">
  <div class="row"><span class="muted small">Click a key to change it, then press the new combination (Esc cancels). Each action can have several.</span><span class="spacer"></span><button class="small" onclick={() => (keys = defaultKeybindings())}>Reset all</button></div>
  <table>
    <tbody>
      {#each ACTIONS as a}
        {@const list = keys[a.id] ?? []}
        <tr>
          <td>{a.label}</td>
          <td class="keys">
            {#each [...list, ...(list.length < 2 ? Array(2 - list.length).fill('') : [])] as k, i}
              <span class="key-slot">
                <button class="key" class:waiting={waiting?.id === a.id && waiting.slot === i} class:conflict={!!k && (owners.get(k)?.length ?? 0) > 1} onclick={() => capture(a.id, i)} title={k && (owners.get(k)?.length ?? 0) > 1 ? 'Also used by another action' : ''}>
                  {waiting?.id === a.id && waiting.slot === i ? 'press keys…' : k ? prettyCombo(k) : '+ add'}
                </button>
                {#if k}<button class="ghost small x" onclick={() => remove(a.id, i)} title="Remove">×</button>{/if}
              </span>
            {/each}
            {#if list.length >= 2}<button class="ghost small" onclick={() => capture(a.id, list.length)}>+</button>{/if}
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>
