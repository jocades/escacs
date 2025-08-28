<script lang="ts">
  import Chessboard from "../chess/chessboard.svelte";

  import { Chess, Move } from "chess.js";
  import type { Cursor, MoveNode, State } from "../chess/tree";
  import { onMount } from "svelte";

  const chess = new Chess();

  const s: State = $state({
    fen: chess.fen(),
    lastMove: undefined,
  });

  class Tree {
    cursor: Cursor = $state({ var: 0, num: -1 });
    value: MoveNode[][] = $state([[]]);

    get(v: number, n: number) {
      return this.value[v][n];
    }

    at() {
      return this.get(this.cursor.var, this.cursor.num);
    }

    line() {
      return this.value[this.cursor.var];
    }

    add(move: Move) {
      this.value[this.cursor.var].push({
        id: { var: this.cursor.var, num: this.cursor.num + 1 },
        move,
        prev: { var: this.cursor.var, num: this.cursor.num },
      });
      this.cursor.num++;
    }

    prev() {
      const node = this.at();
      if (!node) return;
      this.cursor.var = node.prev.var;
      this.cursor.num = node.prev.num;
    }

    next() {
      const line = this.line();
      this.cursor.num = Math.min(this.cursor.num + 1, line.length - 1);
    }

    first() {
      this.cursor.var = 0;
      this.cursor.num = -1;
    }

    last() {
      this.cursor.var = 0;
      this.cursor.num = this.value[0].length - 1;
    }

    loadPgn(chess: Chess, pgn: string) {
      chess.loadPgn(pgn);

      this.value[0] = chess.history({ verbose: true }).map((move, i) => ({
        id: { var: 0, num: i },
        move,
        prev: { var: 0, num: i - 1 },
      }));

      this.cursor.num = this.value[0].length - 1;
    }
  }

  const tree = new Tree();

  $effect(() => {
    const node = tree.at();
    const isStart = tree.cursor.var === 0 && tree.cursor.num === -1;

    if (isStart) {
      chess.reset();
    } else {
      if (!node) return;
      chess.load(node.move.after);
    }

    s.fen = chess.fen();
    s.lastMove = isStart ? undefined : [node.move.from, node.move.to];
  });

  function onMove(from: string, to: string) {
    const move = chess.move({ from, to });
    tree.add(move);

    console.log("onMove", {
      cursor: $state.snapshot(tree.cursor),
      tree: $state.snapshot(tree.value),
    });
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

  const pgn = [
    '[Annotator "User"]',
    `[Date "${new Date().toLocaleDateString()}"]`,
    "",
    "1. d4 {comment 1} d5 {comment 1.5}",
    "2. c4 {comment 2}",
  ].join("\n");

  onMount(() => {
    tree.loadPgn(chess, pgn);

    document.addEventListener("keydown", onKeyDown);

    return () => {
      document.removeEventListener("keydown", onKeyDown);
    };
  });
</script>

<main class="container">
  <Chessboard {chess} state={s} {onMove}></Chessboard>
  <div>
    {#each tree.value[0] as node}
      {node.move.san}
    {/each}
  </div>
</main>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  .container {
    margin: 0;
    padding-top: 10vh;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
    text-align: center;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }
  }
</style>
