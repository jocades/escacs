import type { Move } from "chess.js"
import type { Key, Color } from "chessground/types"

export interface Cursor {
  var: number
  num: number
}

export interface MoveNode {
  id: Cursor
  move: Move
  variations?: number[]
  prev: Cursor
}

export interface State {
  fen: string
  lastMove: [Key, Key] | undefined
  orientation: Color
}
