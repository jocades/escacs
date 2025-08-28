<script lang="ts">
  import type { HTMLButtonAttributes } from "svelte/elements";

  let props: HTMLButtonAttributes = $props();
</script>

<button {...props}>{@render props.children?.()}</button>

<style>
  button {
    cursor: pointer;
    border-radius: 8px;
    border: 1px solid transparent;
    color: #0f0f0f;
    padding: 0.6em 1.2em;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
    &:hover {
      border-color: blue;
    }
  }
</style>
