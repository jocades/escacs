<script>
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let value = $state();
  let result = $state();

  async function greet() {
    result = await invoke("test_what");
  }

  onMount(() => {
    greet();
  });
</script>

<section class="w-1/2">
  <Input bind:value />
  <Button onclick={() => greet()}>X</Button>
  {#if result}
    <div>{result}</div>
  {/if}
</section>
