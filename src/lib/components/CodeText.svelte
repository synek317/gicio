<script lang="ts">
  import type { Tok } from '../highlight';
  let { tokens, text }: { tokens: Tok[] | null; text: string } = $props();
</script>

{#if tokens}{#each tokens as t}<span data-col={t.col} style={t.style}>{t.text}</span>{/each}{:else}<span data-col="0">{text}</span>{/if}
