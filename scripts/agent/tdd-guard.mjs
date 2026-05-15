import { spawnSync } from "node:child_process";

const baseRef = getBaseRef();
const files = collectChangedFiles(baseRef);
const diff = [
  run("git", ["diff", `${baseRef}...HEAD`]),
  run("git", ["diff", "--cached"]),
  run("git", ["diff"]),
].join("\n");

const failures = [];
const warnings = [];

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
const orderCheck = getCommitOrderCheck(baseRef);

if (frontendComponents.length > 0 && frontendTests.length === 0) {
  failures.push("React コンポーネント変更に対応する src/__tests__/ の変更が見つかりません。");
}

if (rustSources.length > 0 && !rustTestsChanged) {
  failures.push("Rust 実装変更に対応する #[cfg(test)] / #[test] / src-tauri/tests の変更が見つかりません。");
}

warnings.push(...orderCheck.warnings);

console.log("TDD guard:");
console.log("- coexistence check: completed");
console.log(`- frontend component changes: ${frontendComponents.length}`);
console.log(`- frontend test changes: ${frontendTests.length}`);
console.log(`- rust source changes: ${rustSources.length}`);
console.log(`- rust tests changed: ${rustTestsChanged ? "yes" : "no"}`);
console.log(`- commit-order check: ${orderCheck.checked ? "completed" : "skipped"}`);
console.log("- uncommitted order: not inferable from git diff");

if (warnings.length > 0) {
  console.log("\nTDD guard warnings:");
  for (const warning of warnings) {
    console.log(`- ${warning}`);
  }
}

if (failures.length > 0) {
  console.error("\nTDD guard failed:");
  for (const failure of failures) {
    console.error(`- ${failure}`);
  }
  process.exit(1);
}

console.log("\nTDD guard passed.");

function getCommitOrderCheck(baseRef) {
  const commits = parseCommitFileHistory(baseRef);
  if (commits.length === 0) {
    return { checked: false, warnings: [] };
  }

  const firstFrontendTestCommit = firstCommitIndex(commits, (file) =>
    file.startsWith("src/__tests__/"),
  );
  const firstRustTestCommit = firstCommitIndexWithCommit(commits, (commit, file) => {
    return (
      (file.startsWith("src-tauri/tests/") && file.endsWith(".rs")) ||
      (file.startsWith("src-tauri/src/") && file.endsWith(".rs") && commitAddsRustTest(commit.hash, file))
    );
  });
  const firstFrontendImplementationCommit = firstCommitIndex(
    commits,
    (file) =>
      file.startsWith("src/components/") &&
      file.endsWith(".tsx") &&
      file !== "src/components/RightPanel.tsx" &&
      file !== "src/components/detail/RelationshipGraphPanel.tsx",
  );
  const firstRustImplementationCommit = firstCommitIndex(
    commits,
    (file) =>
      file.startsWith("src-tauri/src/") &&
      file.endsWith(".rs") &&
      !file.endsWith("/mod.rs"),
  );

  const orderWarnings = [];
  if (
    firstFrontendImplementationCommit !== null &&
    firstFrontendTestCommit !== null &&
    firstFrontendImplementationCommit < firstFrontendTestCommit
  ) {
    orderWarnings.push("React 実装の初出コミットがフロントエンドテストの初出コミットより先です。");
  }

  if (
    firstRustImplementationCommit !== null &&
    firstRustTestCommit !== null &&
    firstRustImplementationCommit < firstRustTestCommit
  ) {
    orderWarnings.push("Rust 実装の初出コミットが Rust テストの初出コミットより先です。");
  }

  return { checked: true, warnings: orderWarnings };
}

function parseCommitFileHistory(baseRef) {
  const log = run("git", ["log", "--reverse", "--name-only", "--pretty=format:commit %H", `${baseRef}..HEAD`]);
  const commits = [];
  let current = null;

  for (const rawLine of log.split(/\r?\n/u)) {
    const line = rawLine.trim();
    if (!line) {
      continue;
    }

    if (line.startsWith("commit ")) {
      current = { hash: line.slice("commit ".length), files: [] };
      commits.push(current);
      continue;
    }

    if (current) {
      current.files.push(line.replaceAll("\\", "/"));
    }
  }

  return commits;
}

function firstCommitIndex(commits, predicate) {
  for (let index = 0; index < commits.length; index += 1) {
    if (commits[index].files.some(predicate)) {
      return index;
    }
  }

  return null;
}

function firstCommitIndexWithCommit(commits, predicate) {
  for (let index = 0; index < commits.length; index += 1) {
    if (commits[index].files.some((file) => predicate(commits[index], file))) {
      return index;
    }
  }

  return null;
}

function commitAddsRustTest(commitHash, file) {
  const diffText = run("git", ["show", "--format=", "--no-ext-diff", commitHash, "--", file]);
  return hasRustTestDiff(diffText);
}

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
