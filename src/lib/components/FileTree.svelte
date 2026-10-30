<script lang="ts">
  import { buildTree } from '../tree';
  import { app } from '../state.svelte';
  import FileTreeNode from './FileTreeNode.svelte';

  const root = $derived(buildTree(app.snap?.files ?? []));
</script>

<ul class="tree">
  {#each root.children as node (node.path)}<FileTreeNode {node} depth={0} />{/each}
</ul>
