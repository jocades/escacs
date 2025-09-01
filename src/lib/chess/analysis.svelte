<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs";
  import { SearchIcon, SettingsIcon } from "@lucide/svelte";
  import type { Info } from "./types";
  import { type State } from "./tree.svelte";
  import { cn } from "$lib/utils";

  const { state, info }: { state: State; info?: Info } = $props();
  const score = $derived.by(() =>
    state.turn === "white" ? info?.score?.cp : (info?.score?.cp ?? 0) * -1,
  );
</script>

{#snippet evaluation(top: boolean)}
  <h2
    class={cn(
      "font-bold tracking-tight text-muted-foreground",
      top ? "text-xl" : "text-sm",
    )}
  >
    {#if score !== undefined && score > 0}+{/if}{score}
  </h2>
{/snippet}

<div class="flex flex-col w-[500px] bg-sidebar">
  <Tabs.Root value="analysis">
    <Tabs.List class="w-full">
      <Tabs.Trigger value="analysis">
        <SearchIcon />
        Analysis
      </Tabs.Trigger>
      <Tabs.Trigger value="other">Other</Tabs.Trigger>
    </Tabs.List>
    <Tabs.Content value="analysis" class="px-2">
      <div class="flex justify-between">
        <h2 class="font-bold">Stockfish 16</h2>
        <SettingsIcon class="size-4.5 cursor-pointer" />
      </div>
      {@render evaluation(true)}
      {#if info !== undefined}
        <div class="flex gap-2">
          {@render evaluation(false)}
          <p class="text-sm truncate">{info.pv.join(" ")}</p>
        </div>
      {/if}
    </Tabs.Content>
    <Tabs.Content value="other">DEF</Tabs.Content>
  </Tabs.Root>
</div>
