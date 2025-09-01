<script lang="ts">
  import type { MoveNode, Tree } from "./tree.svelte";
  import { Button } from "$lib/components/ui/button";
  import {
    ChevronLeftIcon,
    ChevronRightIcon,
    PauseIcon,
    PlayIcon,
  } from "@lucide/svelte";
  import { cn } from "$lib/utils";

  const { tree }: { tree: Tree; info: any } = $props();

  const pairs = $derived.by(() => {
    const result = [];
    for (let i = 0; i < tree.mainLine.length; i += 2) {
      result.push([tree.mainLine[i], tree.mainLine[i + 1]]);
    }
    return result;
  });

  $effect(() => {
    const node = tree.at();
    if (!node) return;
    document
      .querySelector(`#node-${node.id.var}-${node.id.num}`)
      ?.scrollIntoView({
        block: "nearest",
        inline: "nearest",
        behavior: "smooth",
      });
  });
</script>

{#snippet cell(node: MoveNode)}
  <button
    id={`node-${node.id.var}-${node.id.num}`}
    onclick={() => tree.setNode(node)}
    class={cn(
      "cursor-pointer px-0.5",
      node === tree.at() && "bg-zinc-200/20 rounded",
    )}
  >
    {node.move.san}
  </button>
{/snippet}

<div class="flex flex-col h-full max-h-[80vh] max-w-[500px] bg-sidebar">
  <h2 class="px-2 py-2 font-bold">Moves</h2>
  <div class="flex-1 overflow-y-auto">
    <table class="w-full [&_td]:py-1 text-sm">
      <tbody>
        {#each pairs as pair, index}
          <tr class={cn(index % 2 === 0 ? "bg-sidebar" : "bg-zinc-500/10")}>
            <td class="pl-2 font-bold text-muted-foreground">{index + 1}.</td>
            <td>
              {#if pair[0]}{@render cell(pair[0])}{/if}
            </td>
            <td>
              {#if pair[1]}{@render cell(pair[1])}{/if}
            </td>
            <td class="w-1/2"></td>
          </tr>

          {#if pair[0]?.variations}
            {#each pair[0].variations as v}
              <tr class={cn(index % 2 === 0 ? "bg-sidebar" : "bg-zinc-500/10")}>
                <td
                  colspan="4"
                  class="pl-16 text-sm italic text-balance break-words whitespace-normal"
                >
                  (
                  {#each tree.nodes[v] as node, vi}
                    {@render cell(node)}
                    {#if vi < tree.nodes[v].length - 1}{" "}{/if}
                  {/each})
                </td>
              </tr>
            {/each}
          {/if}

          {#if pair[1]?.variations}
            {#each pair[1].variations as v}
              <tr
                class={cn(index % 2 === 0 ? "bg-zinc-500" : "bg-zinc-500/25")}
              >
                <td
                  colspan="4"
                  class="pl-16 italic break-words whitespace-normal"
                >
                  ( ...
                  {#each tree.nodes[v] as node, vi}
                    {@render cell(node)}
                    {#if vi < tree.nodes[v].length - 1}{" "}{/if}
                  {/each}
                  )
                </td>
              </tr>
            {/each}
          {/if}
        {/each}
      </tbody>
    </table>
  </div>
  <div class="flex justify-center py-2">
    <div>
      <Button onclick={() => tree.prev()} size="icon">
        <ChevronLeftIcon />
      </Button>
      <Button onclick={() => tree.play()} size="icon">
        {#if tree.isPlaying}
          <PauseIcon fill="true" />
        {:else}
          <PlayIcon fill="true" />
        {/if}
      </Button>
      <Button onclick={() => tree.next()} size="icon">
        <ChevronRightIcon />
      </Button>
    </div>
  </div>
</div>

<!-- <style> -->
<!--   /* Works in Chrome, Edge, Safari */ -->
<!--   .flex-1::-webkit-scrollbar { -->
<!--     width: 6px; /* thin */ -->
<!--   } -->
<!---->
<!--   .flex-1::-webkit-scrollbar-track { -->
<!--     background: transparent; /* no rectangle */ -->
<!--   } -->
<!---->
<!--   .flex-1::-webkit-scrollbar-thumb { -->
<!--     background-color: black; /* subtle thumb */ -->
<!--     border-radius: 4px; -->
<!--   } -->
<!---->
<!--   /* Works in Firefox */ -->
<!--   .flex-1 { -->
<!--     scrollbar-width: thin; -->
<!--     scrollbar-color: rgba(100, 100, 100, 0.5) transparent; -->
<!--   } -->
<!-- </style> -->
