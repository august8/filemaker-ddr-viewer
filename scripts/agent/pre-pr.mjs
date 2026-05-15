import { spawn, spawnSync } from "node:child_process";

const commandChecks = [
  { label: "frontend tests", command: "npm", args: ["run", "test"] },
  { label: "frontend build", command: "npm", args: ["run", "build"] },
  { label: "rust fmt", command: "cargo", args: ["fmt", "--check"], cwd: "src-tauri" },
  {
    label: "rust clippy",
    command: "cargo",
    args: ["clippy", "--", "-D", "warnings"],
    cwd: "src-tauri",
  },
  { label: "rust tests", command: "cargo", args: ["test"], cwd: "src-tauri" },
];

const warnings = [];
const failures = [];

const changedFiles = getChangedFiles();
const diffText = getCombinedDiff();

console.log("==> scope checks");
console.log(`Changed files: ${changedFiles.length}`);
for (const file of changedFiles) {
  console.log(`- ${file}`);
}

runScopeChecks(changedFiles, diffText);

if (warnings.length > 0) {
  console.log("\nWarnings:");
  for (const warning of warnings) {
    console.log(`- ${warning}`);
  }
}

if (failures.length > 0) {
  console.error("\nPre-PR scope checks failed:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

for (const check of commandChecks) {
  await runCommandCheck(check);
}

console.log("\nAll pre-PR checks passed.");

function runScopeChecks(files, diff) {
  const fileSet = new Set(files);
  const hasFile = (predicate) => files.some(predicate);

  const implementationChanged = hasFile(
    (file) =>
      file.startsWith("src/") ||
      file.startsWith("src-tauri/src/") ||
      file.startsWith("src-tauri/tests/"),
  );
  const frontendComponentChanged = hasFile(
    (file) => file.startsWith("src/components/") && file.endsWith(".tsx"),
  );
  const frontendHookChanged = hasFile((file) => file.startsWith("src/hooks/"));
  const frontendTestChanged = hasFile((file) => file.startsWith("src/__tests__/"));
  const rustSourceChanged = hasFile(
    (file) => file.startsWith("src-tauri/src/") && file.endsWith(".rs"),
  );
  const rustTestChanged =
    hasFile((file) => file.startsWith("src-tauri/tests/") && file.endsWith(".rs")) ||
    hasRustTestDiff(diff);
  const commandChanged = hasFile(
    (file) => file.startsWith("src-tauri/src/commands/") && file.endsWith(".rs"),
  );

  if (implementationChanged && !fileSet.has("ARCHITECTURE.md")) {
    failures.push("実装変更がありますが ARCHITECTURE.md が変更されていません。更新不要な場合は理由を確認してください。");
  }

  if (frontendComponentChanged && !frontendTestChanged) {
    failures.push("React コンポーネント変更がありますが src/__tests__/ の変更がありません。");
  }

  if (rustSourceChanged && !rustTestChanged) {
    failures.push("Rust 実装変更がありますが Rust テスト追加/更新が見つかりません。");
  }

  if (commandChanged && !frontendHookChanged) {
    warnings.push("Tauri command 変更がありますが src/hooks/ の変更がありません。IPC 追加/変更でないか確認してください。");
  }

  if (hasUnfinishedMarker(diff)) {
    failures.push("差分に TODO/FIXME/HACK/todo!()/unimplemented!() が含まれています。");
  }
}

function hasUnfinishedMarker(diff) {
  const markerPattern = /\b(TODO|FIXME|HACK)\b|todo!\s*\(|unimplemented!\s*\(/u;
  let currentFile = "";

  for (const line of diff.split(/\r?\n/u)) {
    const fileMatch = /^diff --git a\/(.+?) b\/(.+)$/u.exec(line);
    if (fileMatch) {
      currentFile = normalizePath(fileMatch[2]);
      continue;
    }

    if (!line.startsWith("+") || line.startsWith("+++")) {
      continue;
    }

    if (!isMarkerCheckedFile(currentFile) || line.includes("markerPattern")) {
      continue;
    }

    if (markerPattern.test(line)) {
      return true;
    }
  }

  return false;
}

function isMarkerCheckedFile(file) {
  return (
    file.startsWith("src/") ||
    file.startsWith("src-tauri/src/") ||
    file.startsWith("src-tauri/tests/") ||
    file.startsWith("tests/") ||
    (file.startsWith("scripts/") && file !== "scripts/agent/pre-pr.mjs")
  );
}

function hasRustTestDiff(diffText) {
  let currentFile = "";
  for (const line of diffText.split(/\r?\n/u)) {
    const fileMatch = /^diff --git a\/(.+?) b\/(.+)$/u.exec(line);
    if (fileMatch) {
      currentFile = normalizePath(fileMatch[2]);
      continue;
    }

    if (!currentFile.startsWith("src-tauri/src/") || !currentFile.endsWith(".rs")) {
      continue;
    }

    if (line.startsWith("+") && /#\[cfg\(test\)\]|#\[test\]/u.test(line)) {
      return true;
    }
  }

  return false;
}

function getChangedFiles() {
  const files = new Set();
  const baseRef = getBaseRef();

  for (const command of [
    ["git", ["diff", "--name-only", `${baseRef}...HEAD`]],
    ["git", ["diff", "--cached", "--name-only"]],
    ["git", ["diff", "--name-only"]],
    ["git", ["ls-files", "--others", "--exclude-standard"]],
  ]) {
    for (const file of runCapture(command[0], command[1]).split(/\r?\n/u)) {
      if (file.trim()) {
        files.add(normalizePath(file.trim()));
      }
    }
  }

  return [...files].sort();
}

function getCombinedDiff() {
  const baseRef = getBaseRef();
  return [
    runCapture("git", ["diff", `${baseRef}...HEAD`]),
    runCapture("git", ["diff", "--cached"]),
    runCapture("git", ["diff"]),
  ].join("\n");
}

function getBaseRef() {
  for (const ref of ["origin/main", "main"]) {
    const result = spawnSync("git", ["rev-parse", "--verify", ref], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
    });
    if (result.status === 0) {
      return ref;
    }
  }

  return "HEAD";
}

function runCapture(command, args) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });

  if (result.status !== 0) {
    return "";
  }

  return result.stdout;
}

function normalizePath(path) {
  return path.replaceAll("\\", "/");
}

function runCommandCheck(check) {
  return new Promise((resolve, reject) => {
    const isWindows = process.platform === "win32";
    const command = isWindows ? process.env.ComSpec || "cmd.exe" : check.command;
    const args = isWindows
      ? ["/d", "/s", "/c", [check.command, ...check.args].map(quoteCmdArg).join(" ")]
      : check.args;

    console.log(`\n==> ${check.label}`);
    console.log(`$ ${[check.command, ...check.args].join(" ")}`);

    const options = { stdio: "inherit" };
    if (check.cwd) {
      options.cwd = check.cwd;
    }

    const child = spawn(command, args, options);

    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) {
        resolve();
        return;
      }

      reject(new Error(`${check.label} failed with exit code ${code}`));
    });
  });
}

function quoteCmdArg(arg) {
  if (!/[\s"]/u.test(arg)) {
    return arg;
  }

  return `"${arg.replaceAll('"', '\\"')}"`;
}
