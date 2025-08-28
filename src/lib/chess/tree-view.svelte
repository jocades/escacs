<script lang="ts">
  import type { MoveNode, Tree } from "./tree.svelte";
  import { Button } from "$lib/components/ui/button";
  import { ChevronLeftIcon, ChevronRightIcon, PlayIcon } from "@lucide/svelte";
  import { cn } from "$lib/utils";

  const { tree }: { tree: Tree } = $props();

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
      node === tree.at() && "bg-background/50 rounded",
    )}
  >
    {node.move.san}
  </button>
{/snippet}

<div class="flex flex-col h-full max-h-[80vh] bg-zinc-700">
  <h2 class="px-2 py-2 font-bold">Moves</h2>
  <div class="flex-1 overflow-y-auto">
    <table class="w-full [&_td]:py-1">
      <tbody>
        {#each pairs as pair, index}
          <tr class={cn(index % 2 === 0 ? "bg-zinc-500" : "bg-zinc-500/25")}>
            <td class="pl-2">{index + 1}.</td>
            <td>
              {#if pair[0]}{@render cell(pair[0])}{/if}
            </td>
            <td>
              {#if pair[1]}{@render cell(pair[1])}{/if}
            </td>
            <td class="w-1/3"></td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
  <div class="flex justify-center py-2">
    <div>
      <Button onclick={() => tree.prev()} size="icon">
        <ChevronLeftIcon />
      </Button>
      <Button size="icon"><PlayIcon fill="true" /></Button>
      <Button onclick={() => tree.next()} size="icon">
        <ChevronRightIcon />
      </Button>
    </div>
  </div>
</div>

<style>
  /* Works in Chrome, Edge, Safari */
  .flex-1::-webkit-scrollbar {
    width: 6px; /* thin */
  }

  .flex-1::-webkit-scrollbar-track {
    background: transparent; /* no rectangle */
  }

  .flex-1::-webkit-scrollbar-thumb {
    background-color: black; /* subtle thumb */
    border-radius: 4px;
  }

  /* Works in Firefox */
  .flex-1 {
    scrollbar-width: thin;
    scrollbar-color: rgba(100, 100, 100, 0.5) transparent;
  }
</style>
