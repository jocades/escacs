const base = "https://lichess.org"
const explorer = "https://explorer.lichess.ovh"

export namespace lch {
  export interface Opening {
    eco: string
    name: string
  }

  export interface Player {
    name: string
    rating?: number
  }

  export interface Analysis {
    inaccuracy: number
    mistake: number
    blunder: number
    acpl: number
    accuracy: number
  }

  export type PlayerWithAnalysis = Player & { analysis: Analysis }

  export type Variant =
    | "standard"
    | "chess960"
    | "crazyhouse"
    | "antichess"
    | "atomic"
    | "horde"
    | "kingOfTheHill"
    | "racingKings"
    | "threeCheck"
    | "fromPosition"

  export type Speed = "ultraBullet" | "bullet" | "blitz" | "rapid" | "classical" | "correspondence"

  export type Status =
    | "created"
    | "started"
    | "aborted"
    | "mate"
    | "resign"
    | "stalemate"
    | "timeout"
    | "draw"
    | "outoftime"
    | "cheat"
    | "noStart"
    | "unknownFinish"
    | "insufficientMaterialClaim"
    | "variantEnd"

  export interface ExplorerMove {
    opening: lch.Opening | null
    averageRating: number
    white: number
    draws: number
    black: number
    uci: string
    san: string
  }

  export interface ExplorerTopGame {
    id: string
    uci: string
    white: lch.Player
    black: lch.Player
    winner: "white" | "black" | null
    year: number
    month: string // `year-month`
  }
}

async function getGames(fen: string) {
  const res = await fetch(`${explorer}/masters?fen=${fen}`)
  return (await res.json()) as {
    opening: lch.Opening | null
    white: number
    draws: number
    black: number
    moves: lch.ExplorerMove[]
    topGames: lch.ExplorerTopGame[]
  }
}

async function getGame(id: string) {
  const res = await fetch(`${base}/game/export/${id}?pgnInJson=true&accuracy=true`, {
    headers: { Accept: "application/json" },
  })
  return (await res.json()) as {
    id: string
    opening: lch.Opening | null
    players: { white: lch.PlayerWithAnalysis; black: lch.PlayerWithAnalysis }
    pgn: string
    moves: string
    rated: boolean
    source: string
    variant: lch.Variant
    speed: lch.Speed
    status: lch.Status
    createdAt: number
    division: { middle: number; end: number }
    import: { date: string }
  }
}

export default { getGames, getGame }
