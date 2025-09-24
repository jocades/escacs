<script lang="ts">
  import "../app.css";
  import { ModeWatcher } from "mode-watcher";
  import Sidebar from "$lib/components/layout/sidebar.svelte";
  import * as Tooltip from "$lib/components/ui/tooltip";

  const { children } = $props();
</script>

<ModeWatcher />
<div class="max-h-screen flex flex-col">
  <Tooltip.Provider>
    <div class="grid grid-cols-[auto_1fr] flex-grow">
      <Sidebar />
      <main class="overflow-x-hidden p-4">
        {@render children()}
      </main>
    </div>
  </Tooltip.Provider>
</div>
