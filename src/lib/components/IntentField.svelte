<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { api, inTauri } from '../api';
  import { app } from '../state.svelte';

  /** Where the reviewer says what the change was supposed to do: typed, pasted or from a file. */
  let { value = $bindable(), id }: { value: string; id: string } = $props();

  async function load() {
    const path = await open({ multiple: false });
    if (typeof path !== 'string') return;
    try {
      value = await api.readFile(path);
    } catch (e) {
      app.toast(String(e), 'error');
    }
  }
</script>

<textarea {id} class="intent-text" bind:value rows="4" placeholder="The task, the prompt you gave the agent or the issue. If this stays empty, the commit messages of the change are used, if there are any."></textarea>
{#if inTauri}<button class="ghost small intent-load" onclick={load}>Load from file…</button>{/if}
