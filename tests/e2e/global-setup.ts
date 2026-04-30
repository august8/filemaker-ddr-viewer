import { spawn } from "child_process";
import path from "path";
import fs from "fs";
import os from "os";
import { fileURLToPath } from "url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));

const BINARY = path.resolve(
  __dirname,
  "../../src-tauri/target/debug/filemaker-ddr-viewer.exe",
);
const STATE_FILE = path.resolve(__dirname, "../../.e2e-state.json");
const CDP_URL = "http://localhost:9222";

async function waitForCDP(timeout: number) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    try {
      const res = await fetch(`${CDP_URL}/json/version`);
      if (res.ok) return;
    } catch {
      // CDP still starting
    }
    await new Promise((r) => setTimeout(r, 300));
  }
  throw new Error(`CDP at ${CDP_URL} did not become available in ${timeout}ms`);
}

export default async function globalSetup() {
  // テスト実行ごとに独立した一時 DB を使用する
  const dbPath = path.join(os.tmpdir(), `fm-e2e-${Date.now()}.db`);

  const proc = spawn(BINARY, [], {
    detached: false,
    stdio: "ignore",
    env: { ...process.env, E2E_DB_PATH: dbPath },
  });

  fs.writeFileSync(STATE_FILE, JSON.stringify({ pid: proc.pid, dbPath }));

  await waitForCDP(20_000);
}
