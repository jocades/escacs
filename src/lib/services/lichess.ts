const base = "https://lichess.org"
const explorer = "https://explorer.lichess.ovh"

namespace lichess {
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
}

async function getGames(fen: string) {
  const res = await fetch(`${explorer}/masters?fen=${fen}`)
  return (await res.json()) as {
    opening: lichess.Opening | null
    white: number
    draws: number
    black: number
    moves: {
      opening: lichess.Opening | null
      averageRating: number
      white: number
      draws: number
      black: number
      uci: string
      san: string
    }[]
    topGames: {
      id: string
      uci: string
      white: lichess.Player
      black: lichess.Player
      winner: "white" | "black"
      year: number
      month: string // `year-month`
    }[]
  }
}

async function getGame(id: string) {
  const res = await fetch(`${base}/game/export/${id}?pgnInJson=true&accuracy=true`, {
    headers: { Accept: "application/json" },
  })
  return (await res.json()) as {
    id: string
    opening: lichess.Opening | null
    players: { white: lichess.PlayerWithAnalysis; black: lichess.PlayerWithAnalysis }
    pgn: string
    moves: string
    rated: boolean
    source: string
    variant: lichess.Variant
    speed: lichess.Speed
    status: lichess.Status
    createdAt: number
    division: { middle: number; end: number }
    import: { date: string }
  }
}

export default { getGames, getGame }
