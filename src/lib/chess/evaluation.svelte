<script lang="ts">
  const { score }: { score: number } = $props();

  const k = 600;

  function clamp(v: number, min: number, max: number) {
    return Math.min(max, Math.max(min, v));
  }

  function sigmoid() {
    return clamp(100 * (1 / (1 + Math.exp(-score / k))), 0, 100);
  }

  const whitePercentage = $derived(sigmoid());
  const blackPercentage = $derived(100 - whitePercentage);

  const label = $derived((score / 100).toFixed(2));
</script>

<div class="flex items-center border">
  <div class="relative h-full w-4 overflow-hidden" role="img">
    <div
      class="absolute bottom-0 left-0 w-full bg-white"
      style={`height:${whitePercentage}%; transition: height 200ms ease`}
    ></div>
    <div
      class="absolute top-0 left-0 w-full bg-black"
      style={`height:${blackPercentage}%; transition: height 200ms ease`}
    ></div>
    <div class="absolute inset-x-0 top-1/2 h-px bg-orange-500"></div>
  </div>
</div>
