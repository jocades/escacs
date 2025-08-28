import { type Chess, SQUARES } from "chess.js"

export function toDests(chess: Chess) {
  const dests = new Map()
  SQUARES.forEach((square) => {
    const moves = chess.moves({ square, verbose: true })
    if (moves.length) {
      dests.set(
        square,
        moves.map((m) => m.to),
      )
    }
  })
  return dests
}

export function toColor(chess: Chess) {
  return chess.turn() == "w" ? "white" : "black"
}
