<script lang="ts">
  import Chessboard from "$lib/chess/chessboard.svelte";
  import TreeView from "$lib/chess/tree-view.svelte";

  import { Chess } from "chess.js";
  import { Tree, type MoveNode, type State } from "$lib/chess/tree.svelte";
  import { onMount } from "svelte";
  import { Channel, invoke } from "@tauri-apps/api/core";
  import Analysis from "$lib/chess/analysis.svelte";
  import Evaluation from "$lib/chess/evaluation.svelte";
  import ipc from "$lib/ipc";
  import { Button } from "$lib/components/ui/button";
  import { longPgn, shortPgn } from "$lib/chess/test-pgns";
  import type { Info, Score } from "$lib/chess/types";
  import { toColor } from "$lib/chess/util";

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

  $inspect(tree.cursor, tree.nodes);

  let engineActive = $state(false);

  let goCount = 0;

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

    // use this to invoke since `s.fen` will trigger causing a invoke call.
    const fen = chess.fen();
    s.fen = fen;
    s.lastMove = isStart ? undefined : [node.move.from, node.move.to];
    s.turn = toColor(chess);
    s.moveNumber = chess.moveNumber();

    if (engineActive) {
      goCount++;
      console.log(goCount);
      // invoke("call_count");
      ipc.go(fen);
      // invoke("go", { fen: chess.fen() });
    }
  });

  function onMove(from: string, to: string) {
    const move = chess.move({ from, to });
    tree.add(move);
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

  const chan = new Channel<Info>();

  let info: Info | undefined = $state();

  function unaryScore(n: number) {
    return chess.turn() === "w" ? n : -n;
  }

  function normalizeScore(score: Score) {
    if (score.cp) score.cp = unaryScore(score.cp);
    else score.mate = unaryScore(score.mate);
  }

  chan.onmessage = (data) => {
    console.log("onMessage", data);
    // normalizeScore(data.score);
    info = data;
  };

  const score = $derived.by(() => {
    if (!info || !info.score?.cp) return 0;
    return chess.turn() === "w" ? info.score?.cp : -info.score?.cp;
  });

  onMount(() => {
    // tree.loadPgn(chess, shortPgn);
    document.addEventListener("keydown", onKeyDown);

    ipc.startEngine(chan).then(() => {
      engineActive = true;
    });

    return () => {
      document.removeEventListener("keydown", onKeyDown);
    };
  });
</script>

<main class="flex h-full justify-center">
  <div class="grid grid-cols-2 gap-x-4">
    <div class="flex flex-col gap-2">
      <div class="flex gap-2 h-[500px]">
        <Evaluation {score} />
        <Chessboard {chess} state={s} {onMove} />
      </div>
      <Button onclick={() => ipc.go(chess.fen())}>Go</Button>
    </div>
    <div class="flex flex-col gap-y-2">
      <Analysis {chess} state={s} {info} />
      <TreeView {tree} {info} />
    </div>
  </div>
</main>
