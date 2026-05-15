import { spawnSync } from "node:child_process";

const baseRef = getBaseRef();
const files = collectChangedFiles(baseRef);
const diff = [
  run("git", ["diff", `${baseRef}...HEAD`]),
  run("git", ["diff", "--cached"]),
  run("git", ["diff"]),
].join("\n");

const failures = [];

const frontendComponents = files.filter(
  (file) =>
    file.startsWith("src/components/") &&
    file.endsWith(".tsx") &&
    file !== "src/components/RightPanel.tsx" &&
    file !== "src/components/detail/RelationshipGraphPanel.tsx",
);
const frontendTests = files.filter((file) => file.startsWith("src/__tests__/"));
const rustSources = files.filter(
  (file) =>
    file.startsWith("src-tauri/src/") &&
    file.endsWith(".rs") &&
    !file.endsWith("/mod.rs"),
);
const rustTestsChanged =
  files.some((file) => file.startsWith("src-tauri/tests/") && file.endsWith(".rs")) ||
  hasRustTestDiff(diff);

if (frontendComponents.length > 0 && frontendTests.length === 0) {
  failures.push("React コンポーネント変更に対応する src/__tests__/ の変更が見つかりません。");
}

if (rustSources.length > 0 && !rustTestsChanged) {
  failures.push("Rust 実装変更に対応する #[cfg(test)] / #[test] / src-tauri/tests の変更が見つかりません。");
}

console.log("TDD guard:");
console.log(`- frontend component changes: ${frontendComponents.length}`);
console.log(`- frontend test changes: ${frontendTests.length}`);
console.log(`- rust source changes: ${rustSources.length}`);
console.log(`- rust tests changed: ${rustTestsChanged ? "yes" : "no"}`);

if (failures.length > 0) {
  console.error("\nTDD guard failed:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("\nTDD guard passed.");

function collectChangedFiles(baseRef) {
  const files = new Set();
  for (const args of [
    ["diff", "--name-only", `${baseRef}...HEAD`],
    ["diff", "--cached", "--name-only"],
    ["diff", "--name-only"],
    ["ls-files", "--others", "--exclude-standard"],
  ]) {
    for (const file of run("git", args).split(/\r?\n/u)) {
      if (file.trim()) {
        files.add(file.trim().replaceAll("\\", "/"));
      }
    }
  }

  return [...files].sort();
}

function getBaseRef() {
  for (const ref of ["origin/main", "main"]) {
    if (spawnSync("git", ["rev-parse", "--verify", ref], { stdio: "ignore" }).status === 0) {
      return ref;
    }
  }

  return "HEAD";
}

function run(command, args) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
  });
  return result.status === 0 ? result.stdout : "";
}

function hasRustTestDiff(diffText) {
  let currentFile = "";
  for (const line of diffText.split(/\r?\n/u)) {
    const fileMatch = /^diff --git a\/(.+?) b\/(.+)$/u.exec(line);
    if (fileMatch) {
      currentFile = fileMatch[2].replaceAll("\\", "/");
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
