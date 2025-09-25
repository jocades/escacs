<script lang="ts">
  import Chessboard from "$lib/components/chess/chessboard.svelte";
  import TreeViewTable from "$lib/components/chess/tree-view-table.svelte";

  import { Chess } from "chess.js";
  import { Tree } from "$lib/chess/tree.svelte";
  import { onMount } from "svelte";
  import { Channel, invoke } from "@tauri-apps/api/core";
  import Analysis from "$lib/components/chess/analysis.svelte";
  import Evaluation from "$lib/components/chess/evaluation.svelte";
  import ipc from "$lib/ipc";
  import { Button } from "$lib/components/ui/button";
  import { longPgn, shortPgn } from "$lib/chess/test-pgns";
  import type {
    Info,
    Opening,
    Score,
    StockfishSettings,
  } from "$lib/chess/types";
  import { playMoveSound, toColor } from "$lib/chess/util";
  import { fetch } from "@tauri-apps/plugin-http";
  import lichess from "$lib/services/lichess";
  import type { BoardState } from "$lib/chess/state.svelte";
  import Notation from "$lib/components/chess/notation.svelte";

  const tree = new Tree();
  const chess = new Chess();
  const boardState: BoardState = $state({
    fen: chess.fen(),
    turn: toColor(chess),
    orientation: "white",
    moveNumber: chess.moveNumber(),
    width: 744,
  });

  const chan = new Channel<Info>();
  const infos: Info[] = $state([]);
  let engineActive = $state(false);

  chan.onmessage = (data) => {
    infos[data.multipv - 1] = data;
  };

  onMount(() => {
    tree.loadPgn(chess, longPgn);
    // ipc.startEngine(chan).then(() => {
    //   engineActive = true;
    // });

    tree.bind();
    return () => {
      tree.unbind();
    };
  });

  $effect(() => {
    const node = tree.at();
    const isStart = tree.isStart();

    if (isStart) {
      chess.reset();
    } else {
      if (!node) return;
      chess.load(node.move.after);
      // playMoveSound(chess, node);
    }

    // use this to invoke since `s.fen` will trigger causing an invoke call.
    const fen = chess.fen();
    boardState.fen = fen;
    boardState.lastMove = isStart ? undefined : [node.move.from, node.move.to];
    boardState.turn = toColor(chess);
    boardState.moveNumber = chess.moveNumber();

    let timeout: number | undefined = setTimeout(async () => {
      if (engineActive) {
        await ipc.go(fen);
      }
    }, 1000);

    // if (s.moveNumber < 36) {
    //   ipc.findOpening(fen).then((o) => {
    //     opening = o;
    //   });
    // }

    return () => {
      if (timeout !== undefined) {
        clearTimeout(timeout);
        timeout = undefined;
      }
    };
  });

  function onMove(from: string, to: string) {
    tree.add(chess.move({ from, to }));
  }

  const stockfishSettings: StockfishSettings = $state({
    multiPV: 3,
    depth: 26,
  });

  const score = $derived.by(() => {
    const mainInfo = infos[0];
    if (!mainInfo || !mainInfo.score?.cp) return 0;
    return chess.turn() === "w" ? mainInfo.score?.cp : -mainInfo.score?.cp;
  });

  function onInfoClick(pv: number, index: number) {
    for (let i = 0; i <= index; i++) {
      const m = infos[pv - 1].pv[i];
      tree.add(chess.move(m));
    }
  }
</script>

<div class="flex w-full h-full gap-x-4">
  <div class="flex flex-col gap-2">
    <div class="flex gap-2" style:width={boardState.width}>
      <Evaluation {score} />
      <Chessboard {chess} {boardState} {onMove} />
    </div>
  </div>
  <div class="flex flex-col gap-y-2">
    <div class="flex flex-1 bg-red-200"></div>
    <Notation {tree} />
  </div>
</div>
