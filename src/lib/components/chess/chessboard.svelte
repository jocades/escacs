<script lang="ts">
  import "./styles/base.css";
  import "./styles/boards/blue.css";
  import "./styles/pieces/chessbase.css";

  import type { Chess } from "chess.js";
  import { Chessground } from "chessground";
  import type { Api as Board } from "chessground/api";

  import { toColor, toDests } from "$lib/chess/util";

  import { onMount } from "svelte";
  import type { BoardState } from "$lib/chess/state.svelte";
  import { DropdownMenuShortcut } from "../ui/dropdown-menu";

  interface ChessboardProps {
    chess: Chess;
    boardState: BoardState;
    onMove?: (from: string, to: string) => void;
  }

  let { chess, boardState, onMove }: ChessboardProps = $props();

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

    const resize = document.createElement("cg-resize");
    resize.onmousedown = (e) => {
      e.stopPropagation();
      const bounds = cg.state.dom.bounds();
      const startX = e.clientX;
      const startY = e.clientY;

      const mouseMove = (e: MouseEvent) => {
        resize.className = "active";
        const width = bounds.width + e.clientX - startX;
        const height = bounds.height + e.clientY - startY;
        boardState.width = Math.round(Math.min(width, height) / 8) * 8;
      };

      const mouseUp = () => {
        resize.removeAttribute("class");
        document.removeEventListener("mousemove", mouseMove);
        document.removeEventListener("mouseup", mouseUp);
      };

      document.addEventListener("mousemove", mouseMove);
      document.addEventListener("mouseup", mouseUp);
    };

    document.querySelector("cg-board")!.appendChild(resize);
  });

  $effect(() => {
    cg.set({
      fen: boardState.fen,
      turnColor: boardState.turn,
      movable: { dests: toDests(chess) },
      check: chess.isCheck(),
      lastMove: boardState.lastMove,
      orientation: boardState.orientation,
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

<div id="board" style:--boardWidth={`${boardState.width}px`}></div>

<style>
  #board {
    width: var(--boardWidth);
    height: var(--boardWidth);
  }
</style>
