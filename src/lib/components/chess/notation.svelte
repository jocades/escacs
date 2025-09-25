<script lang="ts">
  import type { MoveNode, Tree } from "$lib/chess/tree.svelte";
  import { capitalize, cn } from "$lib/utils";
  import { Button, buttonVariants } from "../ui/button";
  import { nag } from "$lib/chess/util";
  import * as Tooltip from "$lib/components/ui/tooltip";
  import * as Popover from "$lib/components/ui/popover";
  import {
    DownloadIcon,
    MessageCircleIcon,
    SaveIcon,
    UploadCloudIcon,
    UploadIcon,
  } from "@lucide/svelte";
  import Textarea from "../ui/textarea/textarea.svelte";
  import Separator from "../ui/separator/separator.svelte";
  import { invoke } from "@tauri-apps/api/core";

  const { tree }: { tree: Tree } = $props();

  function toggler(init = false) {
    let _v = $state(init);

    return {
      get value() {
        return _v;
      },
      set value(v) {
        _v = v;
      },
      toggle() {
        _v = !_v;
      },
    };
  }

  const messageOpen = toggler(false);
</script>

{#snippet moveNumber(n: number)}
  <span class="font-bold text-muted-foreground">{n}.</span>
{/snippet}

{#snippet linebreak()}
  <div class="basis-[100%] h-0"></div>
{/snippet}

{#snippet moveNode(node: MoveNode, depth = 0)}
  <div class={["flex items-center mb-=1"]}>
    <button
      onclick={() => tree.setNode(node)}
      class={[
        "cursor-pointer px-0.5 hover:bg-accent rounded",
        node === tree.at() && "bg-zinc-200/20",
      ]}
      style:color={node.nag?.color}
    >
      {#if node.isWhite}
        <span class="font-bold text-muted-foreground">
          {node.moveNumber}.
        </span>
      {:else if depth > 0 && node.prev.var !== node.id.var}
        <span class="font-bold text-muted-foreground">
          {node.moveNumber}...
        </span>
      {/if}
      {node.move.san}{node.nag?.text}
    </button>
    {#if node.comment}
      <p class="text-sm text-muted-foreground px-1">{node.comment}</p>
    {/if}
  </div>
  {#if node.variations}
    {#each node.variations as v}
      {@render linebreak()}
      <div
        class="flex flex-wrap border-l border-muted pl-2 ml-2"
        style={`margin-left: calc(var(--spacing) * ${4 * (depth + 1)})`}
      >
        {#each tree.nodes[v] as vnode}
          {@render moveNode(vnode, depth + 1)}
        {/each}
      </div>
      {@render linebreak()}
    {/each}
  {/if}
{/snippet}

<div class="flex flex-2 flex-wrap bg-sidebar p-2 content-start">
  {#each tree.mainLine as node}
    {@render moveNode(node)}
  {/each}
</div>

<div class="flex space-x-2">
  <Popover.Root
    open={messageOpen.value}
    onOpenChange={(v) => {
      messageOpen.value = v;
      if (v) tree.unbind();
      else tree.bind();
    }}
  >
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
    <Popover.Content class="flex flex-col p-2">
      <form
        onsubmit={(e) => {
          e.preventDefault();
          // @ts-ignore
          tree.at().comment = e.target.comment.value;
          console.log($state.snapshot(tree.nodes));
          messageOpen.toggle();
        }}
      >
        <Textarea
          name="comment"
          class="h-64"
          autocapitalize="off"
          autocomplete="off"
          value={tree.at()?.comment}
        />
        <Button type="submit" class="mt-2 self-end" size="sm">Add</Button>
      </form>
    </Popover.Content>
  </Popover.Root>
  <Separator orientation="vertical" />
  {#each Object.entries(nag) as [k, v]}
    <Tooltip.Root>
      <Tooltip.Trigger
        onclick={() => {
          const node = tree.at();
          if (node) node.nag = v;
        }}
        class={buttonVariants({ variant: "outline", size: "icon" })}
      >
        {v.text}
      </Tooltip.Trigger>
      <Tooltip.Content>
        <p>{capitalize(k)}</p>
      </Tooltip.Content>
    </Tooltip.Root>
  {/each}
  <Separator orientation="vertical" />
  <Tooltip.Root>
    <Tooltip.Trigger
      onclick={async () => {
        // console.log(JSON.stringify($state.snapshot(tree.nodes)));
        const study = {
          name: "Jordi's Study",
          treeJson: JSON.stringify($state.snapshot(tree.nodes)),
        };
        const id = await invoke("insert_study", { study });
        console.log({ id });
      }}
      class={buttonVariants({ variant: "outline", size: "icon" })}
    >
      <!-- <SaveIcon /> -->
      <!-- <UploadIcon /> -->
      <UploadCloudIcon />
    </Tooltip.Trigger>
    <Tooltip.Content>
      <p>Upload</p>
    </Tooltip.Content>
  </Tooltip.Root>

  <Tooltip.Root>
    <Tooltip.Trigger
      onclick={async () => {
        /* const study = {
          name: "Jordi's Study",
          treeJson: JSON.stringify($state.snapshot(tree.nodes)),
        };
        const id = await invoke("insert_study", { study });
        console.log({ id }); */
      }}
      class={buttonVariants({ variant: "outline", size: "icon" })}
    >
      <DownloadIcon />
    </Tooltip.Trigger>
    <Tooltip.Content>
      <p>Download</p>
    </Tooltip.Content>
  </Tooltip.Root>
</div>
