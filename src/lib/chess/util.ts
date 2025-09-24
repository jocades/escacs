import { type Chess, SQUARES } from "chess.js"
import type { MoveNode } from "./tree.svelte"

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

export const pieceChars = { king: "♔", queen: "♕", rook: "♖", bishop: "♗", knight: "♘" }

export const nag = {
  brilliant: { text: "!!", color: "cyan" },
  good: { text: "!", color: "teal" },
  interesting: { text: "!?", color: "lime" },
  dubious: { text: "?!", color: "yellow" },
  mistake: { text: "?", color: "orange" },
  blunder: { text: "??", color: "red" },
}

const sounds = {
  move: new Audio("sounds/move-self.mp3"),
  capture: new Audio("sounds/move-capture.mp3"),
  check: new Audio("sounds/move-check.mp3"),
  castle: new Audio("sounds/move-castle.mp3"),
  promotion: new Audio("sounds/move-promotion.mp3"),
}

function playSound(kind: keyof typeof sounds) {
  const sound = sounds[kind]
  sound.currentTime = 0
  sound.play()
}

export function playMoveSound(chess: Chess, node: MoveNode) {
  if (chess.inCheck()) {
    playSound("check")
  } else if (node.move.isCapture()) {
    playSound("capture")
  } else if (node.move.isPromotion()) {
    playSound("promotion")
  } else if (node.move.isKingsideCastle() || node.move.isQueensideCastle()) {
    playSound("castle")
  } else {
    playSound("move")
  }
}
