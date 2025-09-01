import { type Channel, invoke } from "@tauri-apps/api/core"
import type { Info } from "./chess/types"

async function startEngine(chan: Channel<Info>) {
  await invoke("start_engine", { chan })
}

async function go(fen: string) {
  await invoke("go", { fen })
}

export default { startEngine, go }
