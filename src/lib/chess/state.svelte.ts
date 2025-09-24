import { Chess } from "chess.js"
import type { Key, Color } from "chessground/types"
import { toColor } from "./util"
import { Tree } from "./tree.svelte"

export interface BoardState {
  fen: string
  turn: Color
  orientation: Color
  moveNumber: number
  lastMove?: [Key, Key]
  width: number
}
