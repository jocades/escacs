<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs";
  import { SearchIcon, SettingsIcon } from "@lucide/svelte";
  import type { Info } from "./types";
  import { type State } from "./tree.svelte";
  import { cn } from "$lib/utils";
  import { Separator } from "$lib/components/ui/separator";
  import type { Chess } from "chess.js";

  const {
    state: s,
    chess,
    info,
  }: { state: State; chess: Chess; info?: Info } = $props();

  const score = $derived.by(
    () =>
      ((s.turn === "white" ? info?.score?.cp : (info?.score?.cp ?? 0) * -1) ??
        0) / 100,
  );

  let engineSettingsActive = $state(false);
</script>

{#snippet evaluation(top: boolean)}
  <h2
    class={cn(
      "font-bold tracking-tight text-zinc-200/80 select-none",
      top ? "text-xl" : "text-sm",
    )}
  >
    {#if score !== undefined && score > 0}+{/if}{score.toFixed(2)}
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
      <div class="flex items-center justify-between">
        <h2 class="font-bold">Stockfish 16</h2>
        <SettingsIcon
          onclick={() => (engineSettingsActive = !engineSettingsActive)}
          class={cn(
            "size-4.5 cursor-pointer rounded-full",
            engineSettingsActive && "text-green-200",
          )}
        />
      </div>
      {@render evaluation(true)}
      <Separator class="my-2" />
      {#if engineSettingsActive}
        <div class="flex">FORM</div>
      {:else}
        <div class="flex gap-2">
          {#if info !== undefined}
            {@render evaluation(false)}
            <p class="text-sm truncate">
              <span class="font-bold text-muted-foreground">
                {s.moveNumber}{#if s.turn === "white"}.{:else}...{/if}
              </span>
              {info.pv.join(" ")}
            </p>
          {/if}
        </div>
      {/if}
    </Tabs.Content>
    <Tabs.Content value="other">DEF</Tabs.Content>
  </Tabs.Root>
</div>
