<script lang="ts">
  import Sidebar from "$lib/components/layout/sidebar.svelte";
  import "../app.css";
  import { ModeWatcher } from "mode-watcher";
  let { children } = $props();
</script>

<ModeWatcher />
<div class="max-h-screen flex flex-col">
  <div class="grid grid-cols-[auto_1fr] flex-grow">
    <Sidebar />
    <main class="overflow-x-hidden px-8">
      {@render children()}
    </main>
  </div>
</div>
