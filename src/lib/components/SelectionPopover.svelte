<script lang="ts">
  import { askAboutSelection, markTarget } from '../actions';
  import { drag } from '../drag.svelte';
  import { parseKey } from '../keys';
  import { app } from '../state.svelte';
  import Composer from './Composer.svelte';

  /** Next to the selected lines, where the pointer is: put them off, reset them, or ask about them. */
  let place = $state<string | null>(null);

  function position() {
    const head = app.selHead && app.selSet.has(app.selHead) ? app.selHead : app.selection.at(-1);
    const k = head ? parseKey(head) : null;
    const row = k ? document.querySelector<HTMLElement>(`[data-line="${CSS.escape(`${k.path}|${k.side}|${k.line}`)}"]`) : null;
    if (!row) return (place = null);
    const r = row.getBoundingClientRect();
    // Below the line the pointer is on; above it when that is the top of the selection.
    const above = head === app.orderedKeys.find((k) => app.selSet.has(k)) && app.selection.length > 1;
    const width = 420;
    const left = Math.max(8, Math.min(r.left + 56, window.innerWidth - width - 8));
    const top = above ? Math.max(8, r.top - 8) : Math.min(r.bottom + 6, window.innerHeight - 60);
    place = `left:${left}px; top:${top}px; width:${width}px;${above ? ' transform: translateY(-100%);' : ''}`;
  }

  $effect(() => {
    void app.selection;
    void app.selHead;
    if (!app.selection.length) return;
    position();
    const sc = document.getElementById('diff-scroll');
    sc?.addEventListener('scroll', position, { passive: true });
    window.addEventListener('resize', position);
    return () => {
      sc?.removeEventListener('scroll', position);
      window.removeEventListener('resize', position);
    };
  });
</script>

{#if app.selection.length && place && !drag.active}
  <div class="sel-pop" style={place}>
    <div class="row">
      <strong>{app.selection.length}</strong> line(s)
      <span class="spacer"></span>
      <button class="small" onclick={() => markTarget('deferred')} title="Put these lines off for later (d)">Later</button>
      <button class="small" onclick={() => markTarget(null)} title="Back to pending (u)">Reset</button>
      <button class="ghost small" onclick={() => (app.selection = [])} title="Clear the selection (Esc)">✕</button>
    </div>
    <Composer send={askAboutSelection} oneLine focus={app.selectionFocus} placeholder="Ask about these lines… (c)" />
  </div>
{/if}
