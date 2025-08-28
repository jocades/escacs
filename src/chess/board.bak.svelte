<script lang="ts">
  import { Chess } from "chess.js";
  import { toColor, toDests } from "../chess/util";
  import { type Cursor, type MoveNode } from "../chess/tree";
  import "../styles/board/base.css";
  import "../styles/board/brown.css";
  import "../styles/board/pieces.css";

  import { Chessground } from "chessground";
  import type { Api as Board } from "chessground/api";
  import { onMount } from "svelte";

  import Button from "../ui/button.svelte";
  import Chessboard from "../chess/chessboard.svelte";

  let cg: Board;
  const chess = new Chess();

  const tree: MoveNode[][] = $state([[]]);
  const c: Cursor = $state({ var: 0, num: -1 });

  const get = (v: number, n: number) => tree[v][n];
  const at = () => get(c.var, c.num);

  const pgn = [
    '[Annotator "User"]',
    `[Date "${new Date().toLocaleDateString()}"]`,
    "",
    "1. d4 {comment 1} d5 {comment 1.5}",
    "2. c4 {comment 2}",
  ].join("\n");

  $effect(() => {
    const node = at();
    if (!cg) return;

    const startpos = c.var === 0 && c.num === -1;

    if (startpos) {
      chess.reset();
    } else {
      if (!node) return;
      chess.load(node.move.after);
    }

    cg.set({
      fen: chess.fen(),
      turnColor: toColor(chess),
      movable: { dests: toDests(chess) },
      check: chess.isCheck(),
      lastMove: startpos ? undefined : [node.move.from, node.move.to],
    });
  });

  function onMove(from: string, to: string) {
    const move = chess.move({ from, to });
    tree[c.var].push({
      id: { var: c.var, num: c.num + 1 },
      move,
      prev: { var: c.var, num: c.num },
    });
    c.num++;

    console.log("onMove", {
      cursor: $state.snapshot(c),
      tree: $state.snapshot(tree),
    });
  }

  function prev() {
    const node = at();
    if (!node) return;
    c.var = node.prev.var;
    c.num = node.prev.num;
  }

  function next() {
    const line = tree[c.var];
    c.num = Math.min(c.num + 1, line.length - 1);
  }

  function onKeyDown(e: KeyboardEvent) {
    switch (e.key) {
      case "ArrowLeft":
        prev();
        break;
      case "ArrowRight":
        next();
        break;
      case "ArrowUp":
        c.var = 0;
        c.num = tree[0].length - 1;
        break;
      case "ArrowDown":
        c.var = 0;
        c.num = -1;
        break;
      default:
        break;
    }
  }

  function loadPgn(pgn: string) {
    chess.loadPgn(pgn);

    tree[0] = chess.history({ verbose: true }).map((move, i) => ({
      id: { var: 0, num: i },
      move,
      prev: { var: 0, num: i - 1 },
    }));

    c.num = tree[0].length - 1;
  }

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

    document.addEventListener("keydown", onKeyDown);

    loadPgn(pgn);

    return () => {
      document.removeEventListener("keydown", onKeyDown);
    };
  });
</script>

<main class="container">
  <Chessboard></Chessboard>
  <!-- <div id="board" style="width: 500px; height: 500px">Hello</div> -->
  <!-- <div> -->
  <!--   {#each tree[0] as node} -->
  <!--     {node.move.san} -->
  <!--   {/each} -->
  <!-- </div> -->
  <!-- <Button onclick={() => console.log({ cursor: c, tree })}>State</Button> -->
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
    text-align: center;
  }

  .logo {
    height: 6em;
    padding: 1.5em;
    will-change: filter;
    transition: 0.75s;
  }

  .logo.tauri:hover {
    filter: drop-shadow(0 0 2em #24c8db);
  }

  .row {
    display: flex;
    justify-content: center;
  }

  a {
    font-weight: 500;
    color: #646cff;
    text-decoration: inherit;
  }

  a:hover {
    color: #535bf2;
  }

  h1 {
    text-align: center;
  }

  input,
  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  }

  button {
    cursor: pointer;
  }

  button:hover {
    border-color: #396cd8;
  }
  button:active {
    border-color: #396cd8;
    background-color: #e8e8e8;
  }

  input,
  button {
    outline: none;
  }

  #greet-input {
    margin-right: 5px;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }

    a:hover {
      color: #24c8db;
    }

    input,
    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }
    button:active {
      background-color: #0f0f0f69;
    }
  }
</style>
