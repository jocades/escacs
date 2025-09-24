<script lang="ts">
  import * as Tabs from "$lib/components/ui/tabs";
  import { DatabaseIcon, SearchIcon, SettingsIcon } from "@lucide/svelte";
  import type { Info, Score } from "$lib/chess/types";
  import { Tree } from "$lib/chess/tree.svelte";
  import { cn } from "$lib/utils";
  import { Separator } from "$lib/components/ui/separator";
  import Database from "./database.svelte";
  import type { Chess } from "chess.js";
  import type { BoardState } from "$lib/chess/state.svelte";

  const {
    boardState,
    chess,
    tree,
    infos,
    onInfoClick,
  }: {
    boardState: BoardState;
    chess: Chess;
    tree: Tree;
    infos: Info[];
    onInfoClick: (pv: number, index: number) => void;
  } = $props();

  const mainInfo = $derived(infos[0]);

  function normScore(score: Score) {
    if (score?.cp === undefined) return 0;
    const cp = score.cp;
    return ((boardState.turn === "white" ? cp : cp * -1) / 100).toFixed(2);
  }

  let engineSettingsActive = $state(false);
</script>

{#snippet evaluation(score: Score, top: boolean)}
  <h2
    class={cn(
      "font-bold tracking-tight text-zinc-200/80",
      top ? "text-xl" : "text-sm",
    )}
  >
    {#if score?.cp !== undefined && score.cp > 0}+{/if}{normScore(score)}
  </h2>
{/snippet}

<div class="flex flex-col w-[500px] bg-sidebar">
  <Tabs.Root value="analysis">
    <Tabs.List class="w-full">
      <Tabs.Trigger value="analysis">
        <SearchIcon />
        Analysis
      </Tabs.Trigger>
      <Tabs.Trigger value="database">
        <DatabaseIcon />
        Database
      </Tabs.Trigger>
    </Tabs.List>
    <Tabs.Content value="analysis" class="px-2 select-none">
      <div class="flex items-center justify-between">
        <h2 class="font-bold">Stockfish 17</h2>
        <SettingsIcon
          onclick={() => (engineSettingsActive = !engineSettingsActive)}
          class={cn(
            "size-4.5 cursor-pointer rounded-full",
            engineSettingsActive && "text-green-200",
          )}
        />
      </div>
      <div class="flex items-center justify-between">
        {@render evaluation(mainInfo?.score, true)}
        <span class="text-sm text-muted-foreground">
          depth={mainInfo?.depth ?? 0}
        </span>
      </div>
      <Separator class="my-2" />
      {#if engineSettingsActive}
        <div class="flex">FORM</div>
      {:else}
        <div class="flex flex-col gap-2">
          {#each infos as info}
            <div class="flex items-center">
              {@render evaluation(info.score, false)}
              <span class="font-bold text-muted-foreground px-2">
                {boardState.moveNumber}{#if boardState.turn === "white"}.{:else}...{/if}
              </span>
              <p class="truncate text-sm space-x-0.5">
                {#each info.pv as san, index}
                  <button
                    onclick={() => onInfoClick(info.multipv, index)}
                    class="cursor-pointer hover:bg-zinc-200/20 rounded px-0.5"
                  >
                    {san}
                  </button>
                {/each}
              </p>
            </div>
          {/each}
        </div>
      {/if}
    </Tabs.Content>
    <Tabs.Content value="database" class="">
      <Database {tree} {chess} {boardState} />
    </Tabs.Content>
  </Tabs.Root>
</div>
