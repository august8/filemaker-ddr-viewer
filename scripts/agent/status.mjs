import { spawnSync } from "node:child_process";

const branch = run("git", ["branch", "--show-current"]).trim() || "(detached)";
const head = run("git", ["log", "-1", "--pretty=format:%h %ad %s", "--date=short"]).trim();
const status = run("git", ["status", "--short"]);
const baseRef = getBaseRef();
const changed = collectChangedFiles(baseRef);

console.log(`Branch: ${branch}`);
console.log(`Base:   ${baseRef}`);
console.log(`HEAD:   ${head}`);
console.log("");

console.log("Changed files:");
if (changed.length === 0) {
  console.log("- none");
} else {
  for (const file of changed) {
    console.log(`- ${file}`);
  }
}

console.log("");
console.log("Git status:");
console.log(status.trim() || "clean");

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
    if (runStatus("git", ["rev-parse", "--verify", ref]) === 0) {
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

function runStatus(command, args) {
  return spawnSync(command, args, { stdio: "ignore" }).status ?? 1;
}

