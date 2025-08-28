import type { Move } from "chess.js"
import type { Key } from "chessground/types"

export interface Cursor {
  var: number
  num: number
}

export interface MoveNode {
  id: Cursor
  move: Move
  lines?: MoveNode[]
  prev: Cursor
}

export interface State {
  fen: string
  lastMove: [Key, Key] | undefined
}
