import { spawn } from "child_process";
import path from "path";
import fs from "fs";
import { fileURLToPath } from "url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));

const BINARY = path.resolve(
  __dirname,
  "../../src-tauri/target/debug/filemaker-ddr-viewer.exe",
);
const PID_FILE = path.resolve(__dirname, "../../.e2e-app.pid");
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
  const proc = spawn(BINARY, [], { detached: false, stdio: "ignore" });
  fs.writeFileSync(PID_FILE, String(proc.pid));
  await waitForCDP(20_000);
}
