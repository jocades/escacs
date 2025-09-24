<script lang="ts">
  import { AnchorIcon, CircuitBoardIcon, type IconProps } from "@lucide/svelte";
  import type { Component } from "svelte";
  import { buttonVariants } from "../ui/button";
  import { page } from "$app/state";
  import * as Tooltip from "$lib/components/ui/tooltip";

  interface SidebarItem {
    title: string;
    icon: Component<IconProps>;
    href: string;
  }

  const navItems: SidebarItem[] = [
    {
      title: "Board",
      icon: CircuitBoardIcon,
      href: "/",
    },
    {
      title: "Studies",
      icon: AnchorIcon,
      href: "/test",
    },
  ];
</script>

<aside
  class="sticky top-0 flex flex-col gap-4 overflow-y-auto py-4 px-2 border-r border-border h-full"
>
  {#each navItems as item}
    <Tooltip.Root>
      <Tooltip.Trigger>
        <a
          href={item.href}
          class={buttonVariants({
            size: "icon",
            variant: page.url.pathname === item.href ? "secondary" : "ghost",
          })}
        >
          <item.icon />
        </a>
      </Tooltip.Trigger>
      <Tooltip.Content side="right">
        <p>{item.title}</p>
      </Tooltip.Content>
    </Tooltip.Root>
  {/each}
</aside>
