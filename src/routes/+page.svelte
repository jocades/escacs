<script lang="ts">
  import Chessboard from "$lib/chess/chessboard.svelte";
  import TreeView from "$lib/chess/tree-view.svelte";

  import { Chess } from "chess.js";
  import { Tree, type MoveNode, type State } from "$lib/chess/tree.svelte";
  import { onMount } from "svelte";

  const chess = new Chess();

  const sounds = {
    move: new Audio("sounds/move-self.mp3"),
    capture: new Audio("sounds/move-capture.mp3"),
    check: new Audio("sounds/move-check.mp3"),
    castle: new Audio("sounds/move-castle.mp3"),
  };

  function playSound(kind: keyof typeof sounds) {
    const sound = sounds[kind];
    sound.pause();
    sound.currentTime = 0;
    sound.play();
  }

  function playMoveSound(node: MoveNode) {
    if (chess.inCheck()) {
      playSound("check");
    } else if (node.move.isCapture()) {
      playSound("capture");
    } else if (node.move.isKingsideCastle() || node.move.isQueensideCastle()) {
      playSound("castle");
    } else {
      playSound("move");
    }
  }

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
      playMoveSound(node);
    }

    _state.fen = chess.fen();
    _state.lastMove = isStart ? undefined : [node.move.from, node.move.to];
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
[Event "Live Chess"]
[Site "Chess.com"]
[Date "2025.08.26"]
[Round "?"]
[White "d4jordi"]
[Black "viny11"]
[Result "1-0"]
[TimeControl "180"]
[WhiteElo "2044"]
[BlackElo "2137"]
[Termination "d4jordi won on time"]
[ECO "D30"]
[EndTime "15:04:42 GMT+0000"]
[Link "https://www.chess.com/game/live/142359911508?move=0"]

1. d4 e6 2. c4 d5 3. Nf3 Nf6 4. g3 dxc4 5. Bg2 c6 6. O-O b5 7. a4 Bb7 8. Qc2 a6
9. Rd1 Qc7 10. Bg5 Be7 11. e4 h6 12. Bxf6 Bxf6 13. d5 cxd5 14. exd5 Bxd5 15.
Rxd5 exd5 16. Qe2+ Be7 17. Nc3 Qd6 18. Re1 Nc6 19. Nxd5 Qxd5 20. Nh4 Qxg2+ 21.
Kxg2 O-O 22. Nf5 Bf6 23. Qf3 Rac8 24. Nd6 Rc7 25. axb5 Nd4 26. Qd5 Rd7 27. Qxc4
Rxd6 28. bxa6 Ra8 29. Ra1 Ra7 30. Qc8+ Kh7 31. Ra4 Ne6 32. b4 Rd4 33. b5 Rxa4
34. Qc2+ g6 35. Qxa4 Bd4 36. Qa5 Re7 37. b6 Ng5 38. b7 Re2 39. Qxg5 Rxf2+ 40.
Kh3 hxg5 41. b8=Q f5 42. Qf8 Bg7 43. Qf7 g4+ 44. Kh4 Rxh2+ 45. Kg5 Ra2 46. a7
Ra6 47. Qb7 Ra2 48. a8=Q Rxa8 49. Qxa8 Bh6+ 50. Kf6 Bg7+ 51. Ke6 g5 52. Qb7 f4
53. Qd5 1-0`;

  onMount(() => {
    // tree.loadPgn(chess, longPgn);

    document.addEventListener("keydown", onKeyDown);

    return () => {
      document.removeEventListener("keydown", onKeyDown);
    };
  });
</script>

<main class="flex h-full justify-center">
  <div class="grid grid-cols-2 gap-x-4">
    <Chessboard {chess} state={_state} {onMove} />
    <TreeView {tree} />
  </div>
</main>
