<script lang="ts">
  import type { MoveNode, Tree } from "$lib/chess/tree.svelte";
  import { capitalize, cn } from "$lib/utils";
  import { Button, buttonVariants } from "../ui/button";
  import { nag } from "$lib/chess/util";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import * as Popover from "$lib/components/ui/popover";
  import { MessageCircleIcon } from "@lucide/svelte";
  import Textarea from "../ui/textarea/textarea.svelte";

  const { tree }: { tree: Tree } = $props();

  const pairs = $derived.by(() => {
    const result = [];
    for (let i = 0; i < tree.mainLine.length; i += 2) {
      result.push([tree.mainLine[i], tree.mainLine[i + 1]]);
    }
    return result;
  });
</script>

{#snippet moveNumber(n: number)}
  <span class="font-bold text-muted-foreground pr-0.5">{n}.</span>
{/snippet}

{#snippet moveNode(node: MoveNode)}
  <button
    onclick={() => tree.setNode(node)}
    class={cn(
      "cursor-pointer px-0.5 hover:bg-accent rounded",
      node === tree.at() && "bg-zinc-200/20",
      // node.nag && `text-${node.nag.color}-500`,
    )}
    style:color={node.nag?.color}
  >
    {node.move.san}{node.nag?.text}
  </button>
  {#if node.comment}
    <p class="text-sm text-muted-foreground px-1">{node.comment}</p>
  {/if}
{/snippet}

<div class="flex flex-2 flex-wrap bg-sidebar p-2 gap-x-2.5 content-start">
  {#each pairs as pair, index}
    <div class="flex items-center">
      {@render moveNumber(index + 1)}
      {#if pair[0]}{@render moveNode(pair[0])}{/if}
      {#if pair[1]}{@render moveNode(pair[1])}{/if}
    </div>
  {/each}
</div>

<div class="flex space-x-2">
  <Popover.Root>
    <Tooltip.Root>
      <Popover.Trigger>
        <Tooltip.Trigger
          class={buttonVariants({ variant: "outline", size: "icon" })}
        >
          <MessageCircleIcon />
        </Tooltip.Trigger>
      </Popover.Trigger>
      <Tooltip.Content>
        <p>Comment</p>
      </Tooltip.Content>
    </Tooltip.Root>
    <Popover.Content class="flex flex-col min-h-64 p-2">
      <form
        onsubmit={(e) => {
          e.preventDefault();
          // @ts-ignore
          tree.at().comment = e.target.comment.value;
          console.log(tree.at());
        }}
      >
        <Textarea
          name="comment"
          class="min-h-64"
          autocapitalize="off"
          autocomplete="off"
          value={tree.at()?.comment}
          onfocusin={() => document.removeEventListener("keydown", tree.bind)}
          onfocusout={() => document.addEventListener("keydown", tree.bind)}
        />
        <Button type="submit" class="mt-2 self-end" size="sm">Add</Button>
      </form>
    </Popover.Content>
  </Popover.Root>
  {#each Object.entries(nag) as [k, v]}
    <Tooltip.Root>
      <Tooltip.Trigger
        onclick={() => (tree.at().nag = v)}
        class={buttonVariants({ variant: "outline", size: "icon" })}
      >
        {v.text}
      </Tooltip.Trigger>
      <Tooltip.Content>
        <p>{capitalize(k)}</p>
      </Tooltip.Content>
    </Tooltip.Root>
  {/each}
</div>
