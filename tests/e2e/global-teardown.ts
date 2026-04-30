import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const STATE_FILE = path.resolve(__dirname, "../../.e2e-state.json");

export default async function globalTeardown() {
  try {
    const { pid, dbPath } = JSON.parse(fs.readFileSync(STATE_FILE, "utf8")) as {
      pid: number;
      dbPath: string;
    };
    process.kill(pid);
    if (fs.existsSync(dbPath)) fs.unlinkSync(dbPath);
    fs.unlinkSync(STATE_FILE);
  } catch {
    // already exited or state file missing
  }
}
