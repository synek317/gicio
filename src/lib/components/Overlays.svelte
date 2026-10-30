<script lang="ts">
  import { settings } from '../app-settings.svelte';
  import { ACTIONS, prettyCombo } from '../keymap';
  import SettingsModal from './SettingsModal.svelte';

  const MOUSE = [
    ['click gutter', 'select a line; shift-click range; ⌘-click toggle; drag to select'],
    ['click • / ✓', 'toggle one line accepted / pending'],
    ['⌘-click code', 'go to definition'],
    ['right-click code', 'definition, references, open file, ask AI'],
    ['Esc', 'clear selection / close'],
  ];
  import { keepHover, leaveCode } from '../lsp';
  import { renderMarkdown } from '../markdown';
  import { app } from '../state.svelte';
  import ContextMenu from './ContextMenu.svelte';
</script>

{#if app.hover}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="hover-card md"
    style="left:{Math.min(app.hover.x + 8, window.innerWidth - 620)}px; top:{app.hover.y + 16}px"
    onmouseenter={keepHover}
    onmouseleave={leaveCode}
  >
    {@html renderMarkdown(app.hover.text)}
  </div>
{/if}

{#if app.menu}<ContextMenu />{/if}

{#if app.showHelp}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="modal-bg" onclick={() => (app.showHelp = false)}>
    <div class="modal help" onclick={(e) => e.stopPropagation()}>
      <h2>Keyboard</h2>
      <table>
        <tbody>
          {#each ACTIONS as a}<tr><td>{#each settings.value.keybindings[a.id] ?? [] as k}<kbd>{prettyCombo(k)}</kbd> {/each}</td><td>{a.label}</td></tr>{/each}
          {#each MOUSE as [k, d]}<tr><td><kbd>{k}</kbd></td><td>{d}</td></tr>{/each}
        </tbody>
      </table>
      <p class="muted small">Change shortcuts in Settings → Shortcuts.</p>
    </div>
  </div>
{/if}

{#if settings.open}<SettingsModal />{/if}

<div class="toasts">
  {#each app.toasts as t (t.id)}
    <div class="toast {t.kind}">{t.text}</div>
  {/each}
</div>
