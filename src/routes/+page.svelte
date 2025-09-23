<script lang="ts">
  import Chessboard from "$lib/components/chess/chessboard.svelte";
  import TreeView from "$lib/components/chess/tree-view.svelte";

  import { Chess } from "chess.js";
  import { Tree, type MoveNode, type State } from "$lib/chess/tree.svelte";
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
  import { toColor } from "$lib/chess/util";
  import { fetch } from "@tauri-apps/plugin-http";
  import lichess from "$lib/services/lichess";

  const chess = new Chess();

  const sounds = {
    move: new Audio("sounds/move-self.mp3"),
    capture: new Audio("sounds/move-capture.mp3"),
    check: new Audio("sounds/move-check.mp3"),
    castle: new Audio("sounds/move-castle.mp3"),
    promotion: new Audio("sounds/move-promotion.mp3"),
  };

  function playSound(kind: keyof typeof sounds) {
    const sound = sounds[kind];
    // sound.pause();
    sound.currentTime = 0;
    sound.play();
  }

  function playMoveSound(node: MoveNode) {
    if (chess.inCheck()) {
      playSound("check");
    } else if (node.move.isCapture()) {
      playSound("capture");
    } else if (node.move.isPromotion()) {
      playSound("promotion");
    } else if (node.move.isKingsideCastle() || node.move.isQueensideCastle()) {
      playSound("castle");
    } else {
      playSound("move");
    }
  }

  const s: State = $state({
    fen: chess.fen(),
    turn: toColor(chess),
    lastMove: undefined,
    orientation: "white",
    moveNumber: chess.moveNumber(),
  });

  const tree = new Tree();

  let engineActive = $state(false);
  let searchDone = $state(false);

  async function search() {
    await ipc.go(chess.fen());
  }

  let goCount = 0;
  let opening: Opening | undefined = $state();

  $effect(() => {
    const node = tree.at();
    const isStart = tree.isStart();

    if (isStart) {
      chess.reset();
    } else {
      if (!node) return;
      chess.load(node.move.after);
      playMoveSound(node);
    }

    // use this to invoke since `s.fen` will trigger causing an invoke call.
    const fen = chess.fen();
    s.fen = fen;
    s.lastMove = isStart ? undefined : [node.move.from, node.move.to];
    s.turn = toColor(chess);
    s.moveNumber = chess.moveNumber();

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

  function onKeyDown(e: KeyboardEvent) {
    switch (e.key) {
      case "ArrowLeft":
        tree.prev();
        break;
      case "ArrowRight":
        tree.next();
        break;
      case "ArrowUp":
        tree.last();
        break;
      case "ArrowDown":
        tree.first();
        break;
      default:
        break;
    }
  }

  const stockfishSettings: StockfishSettings = $state({
    multiPV: 3,
    depth: 26,
  });

  const chan = new Channel<Info>();
  const infos: Info[] = $state([]);

  chan.onmessage = (data) => {
    infos[data.multipv - 1] = data;
    if (
      data.multipv === stockfishSettings.multiPV &&
      data.depth === stockfishSettings.depth
    ) {
      searchDone = true;
    }
  };

  const score = $derived.by(() => {
    const mainInfo = infos[0];
    if (!mainInfo || !mainInfo.score?.cp) return 0;
    return chess.turn() === "w" ? mainInfo.score?.cp : -mainInfo.score?.cp;
  });

  onMount(() => {
    // ipc.startEngine(chan).then(() => {
    //   engineActive = true;
    // });

    document.addEventListener("keydown", onKeyDown);
    return () => {
      document.removeEventListener("keydown", onKeyDown);
    };
  });

  function onInfoClick(pv: number, index: number) {
    for (let i = 0; i <= index; i++) {
      const m = infos[pv - 1].pv[i];
      tree.add(chess.move(m));
    }
  }
</script>

<div class="flex h-full justify-center items-center">
  <div class="grid grid-cols-2 gap-x-4">
    <div class="flex flex-col gap-2">
      <div class="flex gap-2 h-[500px]">
        <Evaluation {score} />
        <Chessboard {chess} state={s} {onMove} />
      </div>
    </div>
    <div class="flex flex-col gap-y-2">
      <Analysis state={s} {chess} {tree} {infos} {onInfoClick} />
      <TreeView {tree} />
    </div>
  </div>
</div>
