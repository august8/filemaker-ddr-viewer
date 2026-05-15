import { existsSync, readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";

const requiredFiles = [
  "AGENTS.md",
  "CLAUDE.md",
  "ARCHITECTURE.md",
  "docs/agent/README.md",
  "docs/agent/CODEX.md",
  "docs/agent/CLAUDE.md",
  "docs/decisions/0012-agent-instruction-split.md",
  "scripts/agent/pre-pr.mjs",
  "scripts/agent/status.mjs",
  "scripts/agent/tdd-guard.mjs",
  "scripts/agent/session-start.mjs",
  "scripts/agent/docs-check.mjs",
];

const scannedFiles = [
  "AGENTS.md",
  "CLAUDE.md",
  "ARCHITECTURE.md",
  "docs/agent/README.md",
  "docs/agent/CODEX.md",
  "docs/agent/CLAUDE.md",
  "docs/decisions/0012-agent-instruction-split.md",
];

const failures = [];

for (const file of requiredFiles) {
  if (!existsSync(file)) {
    failures.push(`Required file is missing: ${file}`);
  }
}

for (const file of scannedFiles) {
  if (!existsSync(file)) {
    continue;
  }
  const content = readFileSync(file, "utf8");
  if (/COMMON\.md|WORKFLOW\.md/u.test(content)) {
    failures.push(`Stale agent doc reference found in ${file}`);
  }
}

const ignored = spawnSync("git", ["check-ignore", "-q", ".codex/hooks.json"], { stdio: "ignore" });
if (ignored.status !== 0) {
  failures.push(".codex/hooks.json is not ignored by git");
}

if (failures.length > 0) {
  console.error("Docs check failed:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("Docs check passed.");

