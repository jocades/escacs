<script>
  import * as Tabs from "$lib/components/ui/tabs";
  import { SearchIcon, SettingsIcon } from "@lucide/svelte";
</script>

<div class="flex flex-col w-full bg-sidebar">
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
    </Tabs.Content>
    <Tabs.Content value="other">DEF</Tabs.Content>
  </Tabs.Root>
</div>
