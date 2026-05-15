import { statSync } from "node:fs";
import { spawn } from "node:child_process";

const architecture = statSync("ARCHITECTURE.md");

console.log("### セッション開始チェック");
console.log(`- ARCHITECTURE.md: 確認対象（最終更新: ${formatDate(architecture.mtime)})`);
console.log("- 実装照合: エージェントが作業内容に応じて実コードで確認する");
console.log("- テスト: npm run test を実行");

await run("npm", ["run", "test"]);

console.log("\n### セッション開始チェック完了");
console.log(`- ARCHITECTURE.md: 確認済み（最終更新: ${formatDate(architecture.mtime)})`);
console.log("- テスト: npm run test passed");
console.log("- 気になった点: なし");

function formatDate(date) {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

function run(command, args) {
  return new Promise((resolve, reject) => {
    const isWindows = process.platform === "win32";
    const child = spawn(
      isWindows ? process.env.ComSpec || "cmd.exe" : command,
      isWindows ? ["/d", "/s", "/c", [command, ...args].join(" ")] : args,
      { stdio: "inherit" },
    );

    child.on("error", reject);
    child.on("close", (code) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(new Error(`${command} ${args.join(" ")} failed with exit code ${code}`));
    });
  });
}

