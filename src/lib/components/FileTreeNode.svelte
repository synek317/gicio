<script lang="ts">
  import { go, markKeys } from '../actions';
  import { changedKeys } from '../keys';
  import { hunksProgress, progressState } from '../progress';
  import { app } from '../state.svelte';
  import type { TreeNode } from '../tree';
  import Self from './FileTreeNode.svelte';

  let { node, depth }: { node: TreeNode; depth: number } = $props();

  const icons = { todo: '○', started: '◐', deferred: '⏸', done: '✓' };
  let open = $state(true);
  const hunks = $derived(node.files.flatMap((f) => f.hunks));
  const p = $derived(hunksProgress(hunks, app.marks));
  const st = $derived(progressState(p));
  const keys = $derived(hunks.flatMap(changedKeys));
  const file = $derived(node.file);
  const active = $derived(!!file && app.view?.kind === 'file' && app.view.path === file.path);
  const paths = $derived(node.files.map((f) => f.path));
  const onScreen = $derived(paths.some((p) => app.onScreen[p]));
  const hovered = $derived(!!app.hoverPath && paths.includes(app.hoverPath));
</script>

<li class="tnode {st}">
  <div class="trow" class:active class:onscreen={onScreen} class:hovered style="padding-left:{depth * 12 + 4}px">
    {#if file}
      <button class="tname" onclick={() => go({ kind: 'file', path: file.path })} title={file.path}>
        <span class="qicon">{icons[st]}</span>
        <span class="status s-{file.status}">{file.status[0].toUpperCase()}</span>
        <span class="tlabel">{node.name}</span>
      </button>
    {:else}
      <button class="tname dir" onclick={() => (open = !open)}>
        <span class="qicon">{open ? '▾' : '▸'}</span><span class="tlabel">{node.name}/</span>
      </button>
    {/if}
    <span class="muted tcount">{file?.binary ? 'bin' : `${p.accepted + p.deferred}/${p.total}`}</span>
    {#if keys.length}
      <span class="factions">
        <button title="Accept everything here" onclick={() => markKeys(keys, 'accepted')}>✓</button>
        <button title="Put everything here off for later" onclick={() => markKeys(keys.filter((k) => app.marks.get(k) !== 'accepted'), 'deferred')}>⏸</button>
      </span>
    {/if}
  </div>
  {#if !file && open}
    <ul class="tree">{#each node.children as child (child.path)}<Self node={child} depth={depth + 1} />{/each}</ul>
  {/if}
</li>
