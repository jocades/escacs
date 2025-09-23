<script lang="ts">
  import "./styles/base.css";
  import "./styles/boards/blue.css";
  import "./styles/pieces/chessbase.css";

  import type { Chess } from "chess.js";
  import { Chessground } from "chessground";
  import type { Api as Board } from "chessground/api";

  import { toColor, toDests } from "$lib/chess/util";
  import type { State } from "$lib/chess/tree.svelte";

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
      drawable: {
        brushes: {
          black: {
            key: "black-arrow",
            color: "#282828",
            opacity: 1,
            lineWidth: 5,
          },
        },
      },
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
      // drawable: {
      //   shapes: [
      //     {
      //       orig: state?.lastMove[0],
      //       dest: state?.lastMove[1],
      //       brush: "black",
      //     },
      //   ],
      // },
    });

    // console.log(cg.state.drawable.shapes);

    // cg.setShapes([
    //   {
    //     orig: "d2",
    //     dest: "d4",
    //     brush: "green",
    //   },
    // ]);
  });
</script>

<div id="board"></div>

<style>
  #board {
    width: 500px;
    height: 500px;
  }
</style>
