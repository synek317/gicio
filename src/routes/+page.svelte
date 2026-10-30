<script lang="ts">
  import { onMount } from 'svelte';
  import { api, onJobs, onSession } from '$lib/api';
  import { handleKey } from '$lib/hotkeys';
  import { pollLspStatus } from '$lib/lsp';
  import { app } from '$lib/state.svelte';
  import Overlays from '$lib/components/Overlays.svelte';
  import ReviewApp from '$lib/components/ReviewApp.svelte';
  import StartScreen from '$lib/components/StartScreen.svelte';
  import { settings } from '$lib/app-settings.svelte';

  let ready = $state(false);
  settings.load().then(() => (ready = true));

  onMount(() => {
    const off = onSession((s) => app.snap && app.applySnapshot(s));
    const stop = pollLspStatus();
    const offJobs = onJobs((j) => (app.jobs = j), (id, line) => app.jobLog(id, line));
    api.listJobs().then((j) => (app.jobs = j)).catch(() => {});
    return () => {
      off();
      offJobs();
      stop();
    };
  });
</script>

<svelte:window onkeydown={handleKey} />

{#if !ready}{:else if app.snap}<ReviewApp />{:else}<StartScreen />{/if}
<Overlays />
