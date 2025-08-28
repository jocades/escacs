import type { Chess, Move } from "chess.js"
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

export class Tree {
  /**
   * main line -> index 0
   * variations -> index N (where N > 0)
   */
  nodes: MoveNode[][] = $state([[]])
  cursor: Cursor = $state({ var: 0, num: -1 })

  get(v: number, n: number) {
    return this.nodes[v][n]
  }

  at() {
    return this.get(this.cursor.var, this.cursor.num)
  }

  get line() {
    return this.nodes[this.cursor.var]
  }

  get mainLine() {
    return this.nodes[0]
  }

  getNext() {
    return this.get(this.cursor.var, this.cursor.num + 1)
  }

  setCursor(v: number, n: number) {
    this.cursor.var = v
    this.cursor.num = n
  }

  setNode(node: MoveNode) {
    this.setCursor(node.id.var, node.id.num)
  }

  add(move: Move) {
    const existing = this.getNext()

    if (!existing) {
      // new move
      this.line.push({
        id: { var: this.cursor.var, num: this.cursor.num + 1 },
        move,
        prev: { var: this.cursor.var, num: this.cursor.num },
      })
      this.cursor.num++
      return
    }

    const toCheck = [existing]
    existing.variations?.forEach((v) => toCheck.push(this.nodes[v][0]))
    const node = toCheck.find((n) => n.move.san === move.san)

    if (node) {
      // existing variation
      this.setCursor(node.id.var, node.id.num)
    } else {
      // new variation
      if (!existing.variations) existing.variations = []
      const v = this.nodes.length
      existing.variations.push(v)
      this.nodes[v] = [
        {
          id: { var: v, num: 0 },
          move,
          prev: existing.prev,
        },
      ]
      this.setCursor(v, 0)
    }
  }

  prev() {
    const node = this.at()
    if (!node) return
    this.setCursor(node.prev.var, node.prev.num)
  }

  next() {
    const node = this.getNext()
    if (!node) return

    this.cursor.num++

    if (node.variations) {
      const choices = [0, ...node.variations]
      const choice = choices[Math.floor(Math.random() * choices.length)]
      console.log({ choices, choice })

      if (this.cursor.var !== choice) {
        this.setCursor(choice, 0)
      }
    }
  }

  first() {
    this.setCursor(0, -1)
  }

  last() {
    this.setCursor(0, this.mainLine.length - 1)
  }

  loadPgn(chess: Chess, pgn: string) {
    chess.loadPgn(pgn)
    this.nodes[0] = chess.history({ verbose: true }).map((move, i) => ({
      id: { var: 0, num: i },
      move,
      prev: { var: 0, num: i - 1 },
    }))
    this.setCursor(0, this.mainLine.length - 1)
  }

  isStart() {
    return this.cursor.var === 0 && this.cursor.num === -1
  }
}
