<script lang="ts">
  import Chessboard from "../chess/chessboard.svelte";

  import { Chess, Move } from "chess.js";
  import type { Cursor, MoveNode, State } from "../chess/tree";
  import { onMount } from "svelte";

  const chess = new Chess();

  const s: State = $state({
    fen: chess.fen(),
    lastMove: undefined,
    orientation: "white",
  });

  class Tree {
    /**
     * main line -> index 0
     * variations -> index N (where N > 0)
     */
    nodes: MoveNode[][] = $state([[]]);
    cursor: Cursor = $state({ var: 0, num: -1 });

    get(v: number, n: number) {
      return this.nodes[v][n];
    }

    at() {
      return this.get(this.cursor.var, this.cursor.num);
    }

    get line() {
      return this.nodes[this.cursor.var];
    }

    get mainLine() {
      return this.nodes[0];
    }

    getNext() {
      return this.get(this.cursor.var, this.cursor.num + 1);
    }

    setCursor(v: number, n: number) {
      this.cursor.var = v;
      this.cursor.num = n;
    }

    add(move: Move) {
      const existing = this.getNext();

      if (!existing) {
        // new move
        console.log("new move");
        this.line.push({
          id: { var: this.cursor.var, num: this.cursor.num + 1 },
          move,
          prev: { var: this.cursor.var, num: this.cursor.num },
        });
        this.cursor.num++;
        return;
      }

      console.log("existing");
      const toCheck = [existing];
      existing.variations?.forEach((v) => toCheck.push(this.nodes[v][0]));
      const node = toCheck.find((n) => n.move.san === move.san);

      if (node) {
        // existing variation
        console.log("existing variation");
        this.setCursor(node.id.var, node.id.num);
      } else {
        // new variation
        console.log("new variation");
        if (!existing.variations) existing.variations = [];
        const v = this.nodes.length;
        existing.variations.push(v);
        this.nodes[v] = [
          {
            id: { var: v, num: 0 },
            move,
            prev: existing.prev,
          },
        ];
        this.setCursor(v, 0);
      }
    }

    prev() {
      const node = this.at();
      if (!node) return;
      this.setCursor(node.prev.var, node.prev.num);
    }

    next() {
      const node = this.getNext();
      if (!node) return;

      this.cursor.num++;

      if (node.variations) {
        const choices = [0, ...node.variations];
        const choice = choices[Math.floor(Math.random() * choices.length)];
        console.log({ choices, choice });

        if (this.cursor.var !== choice) {
          this.setCursor(choice, 0);
        }
      }
    }

    first() {
      this.setCursor(0, -1);
    }

    last() {
      this.setCursor(0, this.mainLine.length - 1);
    }

    loadPgn(chess: Chess, pgn: string) {
      chess.loadPgn(pgn);
      this.nodes[0] = chess.history({ verbose: true }).map((move, i) => ({
        id: { var: 0, num: i },
        move,
        prev: { var: 0, num: i - 1 },
      }));
      this.setCursor(0, this.mainLine.length - 1);
    }

    isStart() {
      return this.cursor.var === 0 && this.cursor.num === -1;
    }
  }

  const tree = new Tree();

  $inspect(tree.cursor, tree.nodes);

  $effect(() => {
    const node = tree.at();
    const isStart = tree.isStart();

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
  <Chessboard {chess} state={s} {onMove} />
  <div>
    {#each tree.nodes[0] as node}
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
