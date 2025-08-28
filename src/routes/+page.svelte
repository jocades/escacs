<script lang="ts">
  import Chessboard from "$lib/chess/chessboard.svelte";
  import TreeView from "$lib/chess/tree-view.svelte";

  import { Chess } from "chess.js";
  import { Tree, type State } from "$lib/chess/tree.svelte";
  import { onMount } from "svelte";

  const chess = new Chess();

  const sounds = {
    move: new Audio("sounds/move-self.mp3"),
  };

  // function playSound(kind: keyof typeof sounds) {
  //   if (!sounds[kind]) return;
  //   const sound = new Audio(sounds[kind]);
  //   sound.play();
  // }

  const _state: State = $state({
    fen: chess.fen(),
    lastMove: undefined,
    orientation: "white",
  });

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

    _state.fen = chess.fen();
    _state.lastMove = isStart ? undefined : [node.move.from, node.move.to];

    sounds.move.currentTime = 0;
    sounds.move.play();
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

  const longPgn = `
[Event "Hourly SuperBlitz Arena"]
[Site "https://lichess.org/55tGMOlU"]
[Date "2025.08.28"]
[White "Meerbos14"]
[Black "admin112"]
[Result "1/2-1/2"]
[GameId "55tGMOlU"]
[UTCDate "2025.08.28"]
[UTCTime "15:55:04"]
[WhiteElo "1992"]
[BlackElo "2050"]
[WhiteRatingDiff "+1"]
[BlackRatingDiff "+0"]
[Variant "Standard"]
[TimeControl "180+0"]
[ECO "B10"]
[Opening "Caro-Kann Defense: Accelerated Panov Attack"]
[Termination "Normal"]

1. e4 c6 2. c4 d5 3. cxd5 cxd5 4. Nc3 Nc6 5. exd5 Nb4 6. Bc4 Bd7 7. Nf3 Nf6 8. O-O g6 9. a3 Na6 10. d4 Bg7 11. d6 e6 12. d5 O-O 13. dxe6 Bxe6 14. Bxe6 fxe6 15. Re1 Re8 16. Ne5 Nc5 17. b4 Ncd7 18. Bb2 Rc8 19. Nb5 Nxe5 20. Bxe5 Qb6 21. d7 Qxb5 22. dxc8=Q Rxc8 23. Bxf6 Bxf6 24. Rxe6 Bxa1 25. Qxa1 Qc4 26. Qe1 Qc1 27. g3 Qxe1+ 28. Rxe1 Rc3 29. Re3 Rc1+ 30. Kg2 Ra1 31. Rc3 a6 32. Kf3 Kg7 33. Rc7+ Kf6 34. Rxb7 Rxa3+ 35. Kf4 Rb3 36. f3 a5 37. h4 Rxb4+ 38. Rxb4 axb4 39. Ke3 Kf5 40. Kd3 h5 41. Kc4 Ke5 42. Kxb4 Kd4 43. Kb5 Ke3 44. g4 Kf4 45. gxh5 gxh5 46. Kc4 Kxf3 47. Kd3 Kg3 48. Ke3 Kxh4 49. Kf2 Kh3 50. Kf3 Kh2 51. Kf2 h4 52. Kf1 h3 53. Kf2 Kh1 54. Kf1 Kh2 55. Kf2 Kh1 56. Kf1 Kh2 1/2-1/2`;

  onMount(() => {
    tree.loadPgn(chess, longPgn);

    document.addEventListener("keydown", onKeyDown);

    return () => {
      document.removeEventListener("keydown", onKeyDown);
    };
  });
</script>

<main class="min-h-[100vh] flex items-center justify-center">
  <div class="grid grid-cols-2 gap-x-4">
    <Chessboard {chess} state={_state} {onMove} />
    <TreeView {tree} />
  </div>
</main>
