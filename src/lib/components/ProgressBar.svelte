<script lang="ts">
  import type { Progress } from '../progress';
  let { p, thin = false }: { p: Progress; thin?: boolean } = $props();
  const pct = (n: number) => (p.total ? (100 * n) / p.total : 0);
</script>

<div class="pbar" class:thin>
  <span class="acc" style="width:{pct(p.accepted)}%"></span><span class="def" style="width:{pct(p.deferred)}%"></span>
</div>
