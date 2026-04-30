import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const PID_FILE = path.resolve(__dirname, "../../.e2e-app.pid");

export default async function globalTeardown() {
  try {
    const pid = parseInt(fs.readFileSync(PID_FILE, "utf8"), 10);
    process.kill(pid);
    fs.unlinkSync(PID_FILE);
  } catch {
    // already exited
  }
}
