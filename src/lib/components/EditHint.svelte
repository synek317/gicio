<script lang="ts">
  let { checked = $bindable() }: { checked: boolean } = $props();
</script>

<label
  class="toggle"
  title="On: when you ask for a change (or the AI finds a real bug while answering), it edits files in your working tree right away. It never commits. The diff and the queue update afterwards.&#10;Off: the AI only answers and does not touch any file."
>
  <input type="checkbox" bind:checked /> AI may edit code <span class="info">ⓘ</span>
</label>
