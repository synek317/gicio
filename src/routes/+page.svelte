<script lang="ts">
  import { onMount } from 'svelte';
  import { api, onJobs, onSession, onWorktreeChanged } from '$lib/api';
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
    const offDisk = onWorktreeChanged((files) => {
      const list = files.slice(0, 3).join(', ') + (files.length > 3 ? ` and ${files.length - 3} more` : '');
      app.toast(`Changed on disk: ${list}. The diff is updated; new code is in a step at the end.`, 'info', 6000);
    });
    return () => {
      off();
      offJobs();
      offDisk();
      stop();
    };
  });
</script>

<svelte:window onkeydown={handleKey} />

{#if !ready}{:else if app.snap}<ReviewApp />{:else}<StartScreen />{/if}
<Overlays />
