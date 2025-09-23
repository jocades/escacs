<script>
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let value = $state();
  let result = $state();

  async function greet() {
    // result = await invoke("test_what");
  }

  onMount(() => {
    greet();
  });

  const items = Array.from({ length: 50 }, (_, i) => i);
</script>

<div class="flex flex-col gap-4">
  {#each items as item}
    <div class="size-20 bg-blue-200 flex items-center justify-center">
      {item}
    </div>
  {/each}
</div>
