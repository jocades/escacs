<script lang="ts">
  import type { lch } from "$lib/services/lichess";
  import lichess from "$lib/services/lichess";
  import * as Table from "$lib/components/ui/table/index";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index";
  import type { Tree } from "$lib/chess/tree.svelte";
  import type { Chess } from "chess.js";
  import { Separator } from "$lib/components/ui/separator/index";
  import { cn } from "$lib/utils";
  import { SquareIcon } from "@lucide/svelte";
  import { Input } from "$lib/components/ui/input";
  import ipc from "$lib/ipc";
  import type { BoardState } from "$lib/chess/state.svelte";

  const {
    tree,
    chess,
    boardState,
  }: {
    tree: Tree;
    chess: Chess;
    boardState: BoardState;
  } = $props();

  let games: Awaited<ReturnType<typeof lichess.getGames>> | undefined =
    $state();

  function debounce<T>(cb: (...args: T[]) => any, ms: number) {
    let timeout: number | undefined;

    return (...args: T[]) => {
      clearTimeout(timeout);
      timeout = setTimeout(() => cb(...args), ms);
    };
  }

  // const update = debounce(async (fen: string) => {
  //   games = await lichess.getGames(fen);
  // }, 500);

  // $effect(() => update(s.fen));

  // debounce(async () => {
  //   const fen = s.fen;
  //   games = await lichess.getGames(fen);
  // }, 500);

  /* $effect(() => {
    const fen = s.fen;
    let timeout: number | undefined = setTimeout(async () => {
      console.log("get games");
      games = await lichess.getGames(fen);
    }, 500);
    return () => {
      if (timeout !== undefined) {
        clearTimeout(timeout);
        timeout = undefined;
      }
    };
  }); */

  let filteredPlayer = $state("");
  const filteredTopGames = $derived.by(() => {
    if (!filteredPlayer) {
      return games?.topGames ?? [];
    }

    return games?.topGames.filter(
      (g) =>
        g.white.name.toLowerCase().includes(filteredPlayer.toLowerCase()) ||
        g.black.name.toLowerCase().includes(filteredPlayer.toLowerCase()),
    );
  });

  async function loadGame(id: string) {
    const game = await lichess.getGame(id);
    // await ipc.newGame();
    tree.loadPgn(chess, game.pgn);
  }
</script>

<ScrollArea class="h-[250px]">
  <h2 class="font-bold tracking-tight px-2">Top Moves</h2>
  <Table.Root class="text-xs">
    <Table.Header>
      <Table.Row>
        <Table.Head>Move</Table.Head>
        <Table.Head>Games</Table.Head>
        <Table.Head>Rating</Table.Head>
        <Table.Head>Percentages</Table.Head>
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#if games !== undefined}
        {#each games.moves as move}
          <Table.Row
            class="cursor-pointer border-zinc-500/80"
            onclick={() => tree.add(chess.move(move.uci))}
          >
            <Table.Cell>
              {move.san}
            </Table.Cell>
            <Table.Cell class="">
              {move.white + move.draws + move.black}
            </Table.Cell>
            <Table.Cell>
              {move.averageRating}
            </Table.Cell>
            <Table.Cell>
              {(
                (move.white / (move.white + move.draws + move.black)) *
                100
              ).toFixed(0)}% /
              {(
                (move.draws / (move.white + move.draws + move.black)) *
                100
              ).toFixed(0)}% /
              {(
                (move.black / (move.white + move.draws + move.black)) *
                100
              ).toFixed(0)}%
            </Table.Cell>
          </Table.Row>
        {/each}
      {/if}
    </Table.Body>
  </Table.Root>
  <Separator class="my-2" />
  <div class="flex flex-col">
    <div class="flex items-center justify-between pb-2 px-2">
      <h2 class="font-bold tracking-tight">Top Games</h2>
      <Input
        placeholder="Filter player..."
        autocorrect="off"
        class="w-64 h-6"
        bind:value={filteredPlayer}
      />
    </div>
    {#if filteredTopGames !== undefined}
      <div class="flex flex-col">
        {#each filteredTopGames as topGame, index}
          <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
          <div
            onclick={async () => await loadGame(topGame.id)}
            class={cn(
              "grid grid-cols-4 items-center text-xs p-2 hover:bg-accent cursor-pointer",
              index % 2 === 0 ? "bg-sidebar" : "bg-zinc-500/10",
            )}
          >
            <div class="flex flex-col col-span-2">
              <div>
                {topGame.white.name}
                <span class="text-muted-foreground">
                  ({topGame.white.rating})
                </span>
              </div>
              <div>
                {topGame.black.name}
                <span class="text-muted-foreground">
                  ({topGame.black.rating})
                </span>
              </div>
            </div>
            {#if topGame.winner}
              <SquareIcon class="size-3.5" fill={topGame.winner} />
            {:else}
              <h1>½</h1>
            {/if}
            <div>{topGame.month}</div>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</ScrollArea>
