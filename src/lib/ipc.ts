import { type Channel, invoke } from "@tauri-apps/api/core"
import type { Info, Opening } from "./chess/types"

async function startEngine(chan: Channel<Info>) {
  await invoke("start_engine", { chan })
}

async function go(fen: string) {
  await invoke("go", { fen })
}

async function newGame() {
  await invoke("new_game")
}

async function findOpening(fen: string): Promise<Opening | undefined> {
  return await invoke("find_opening", { fen })
}

export default { startEngine, go, newGame, findOpening }
