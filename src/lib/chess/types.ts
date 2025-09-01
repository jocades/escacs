export type Score = { cp: number; mate: never } | { mate: number; cp: never }

export interface Info {
  depth: number
  seldepth: number
  multipv: number
  score: Score
  wdl: [number, number, number]
  nodes: number
  nps: number
  hashfull: number
  tbhits: number
  time: number
  pv: string[]
}
