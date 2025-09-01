<script lang="ts">
  import "./styles/base.css";
  import "./styles/brown.css";
  import "./styles/pieces.css";

  import type { Chess } from "chess.js";
  import { Chessground } from "chessground";
  import type { Api as Board } from "chessground/api";

  import { toColor, toDests } from "../chess/util";
  import type { State } from "../chess/tree.svelte";

  import { onMount } from "svelte";

  interface ChessboardProps {
    chess: Chess;
    state: State;
    onMove?: (from: string, to: string) => void;
  }

  let { chess, state, onMove }: ChessboardProps = $props();

  let cg: Board;
  onMount(() => {
    cg = Chessground(document.querySelector("#board")!, {
      movable: {
        free: false,
        dests: toDests(chess),
        events: { after: onMove },
      },
      highlight: { check: true },
      draggable: { showGhost: false },
    });
  });

  $effect(() => {
    cg.set({
      fen: state.fen,
      turnColor: state.turn,
      movable: { dests: toDests(chess) },
      check: chess.isCheck(),
      lastMove: state.lastMove,
      orientation: state.orientation,
    });
  });
</script>

<div id="board"></div>

<style>
  #board {
    width: 500px;
    height: 500px;
  }
</style>
