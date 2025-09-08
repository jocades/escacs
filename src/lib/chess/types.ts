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

export interface StockfishSettings {
  /**
   * `type spin default 1 min 1 max 1024`
   *
   * The number of CPU threads used for searching a position. For best performance,
   * set this equal to the number of CPU cores available.
   */
  threads?: number
  /**
   * `type spin default 16 min 1 max 33554432`
   *
   * The size of the hash table in MB. It is recommended to set Hash after setting Threads.
   */
  hash?: number
  /**
   * `type spin default 1 min 1 max 500`
   *
   * Output the N best lines (principal variations, PVs) when searching. Leave at 1 for the best performance.
   *
   */
  multiPV?: number
  /**
   * `type check default false`
   *
   * If true, Stockfish will play Chess960.
   */
  chess960?: boolean
  /**
   * `type check default false`
   *
   * If enabled, show approximate WDL statistics as part of the engine output.
   */
  showWDL?: boolean
  /**
   * `type check default false`
   *
   * Enable weaker play aiming for an Elo rating as set by UCI_Elo. This option overrides Skill Level.
   */
  limitStrength?: boolean
  /**
   * `type spin default 1320 min 1320 max 3190`
   *
   * If UCI_LimitStrength is enabled, it aims for an engine strength of the given Elo. This Elo rating has been calibrated at a time control of 120s+1s and anchored to CCRL 40/4.
   */
  elo?: number
  /*
   * `type spin default 20 min 0 max 20`
   *
   * Lower the Skill Level in order to make Stockfish play weaker (see also UCI_LimitStrength).
   * Internally, MultiPV is enabled, and with a certain probability depending on the Skill Level, a weaker move will be played.
   */
  skillLevel?: number
  depth?: number
}
