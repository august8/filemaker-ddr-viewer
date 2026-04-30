import path from "path";
import { fileURLToPath } from "url";
import { test, expect } from "./fixtures";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const DDR_SUMMARY = path.resolve(__dirname, "../ddr/22.0.6.601/概要.xml");

type TauriInternals = {
  __TAURI_INTERNALS__: { invoke: (cmd: string, args: unknown) => Promise<unknown> };
};

// テスト 2〜4 はテスト 1 のインポート結果に依存するため serial を指定する
test.describe.serial("Golden path: DDR import → 検索 → 詳細表示", () => {
  test("1. import_ddr_from_path でソリューションをインポートできる", async ({ page }) => {
    // Tauri IPC 準備完了まで待機
    await page.waitForFunction(
      () => !!(window as unknown as TauriInternals).__TAURI_INTERNALS__?.invoke,
      { timeout: 15_000 },
    );

    const result = await page.evaluate(async (summaryPath: string) => {
      try {
        return await (window as unknown as TauriInternals).__TAURI_INTERNALS__.invoke(
          "import_ddr_from_path",
          { summaryPath },
        );
      } catch (e) {
        return { _error: (e as Error).message };
      }
    }, DDR_SUMMARY);

    const res = result as { _error?: string; solution: unknown; projects: unknown[] };
    expect(res._error).toBeUndefined();
    expect(res.solution).toBeTruthy();
    expect(res.projects.length).toBeGreaterThan(0);

    // React Query キャッシュを無効化してサイドバーを再フェッチさせる
    type QC = { invalidateQueries: (arg: { queryKey: string[] }) => void };
    await page.evaluate(() => {
      (window as unknown as { __queryClient?: QC }).__queryClient
        ?.invalidateQueries({ queryKey: ["solutions"] });
    });
    // waitForTimeout は使わず、ステップ 2 で実際の表示を待つ
  });

  test("2. サイドバーにソリューション名が表示される", async ({ page }) => {
    const solutions = await page.evaluate(() =>
      (window as unknown as TauriInternals).__TAURI_INTERNALS__.invoke(
        "list_solutions",
        {},
      ),
    ) as Array<{ name: string }>;

    expect(solutions.length).toBeGreaterThan(0);
    const solutionName = solutions[0].name;

    await expect(
      page.locator("aside").getByText(solutionName, { exact: false }).first(),
    ).toBeVisible({ timeout: 15_000 });
  });

  test("3. 検索バーでキーワード検索できる", async ({ page }) => {
    await page.getByPlaceholder(/検索/).fill("BaseFile");
    // FTS5 デバウンス完了後に結果が出るまでポーリングで待つ（固定スリープ不要）
    await expect(
      page.locator('[data-testid="search-result-item"]').first(),
    ).toBeVisible({ timeout: 5_000 });
  });

  test("4. 検索結果クリックで詳細パネルが表示される", async ({ page }) => {
    await page.locator('[data-testid="search-result-item"]').first().click();
    // クリックで searchQuery がクリアされ、詳細パネル（data-testid="detail-panel"）が出現する
    await expect(
      page.locator('[data-testid="detail-panel"]'),
    ).toBeVisible({ timeout: 10_000 });
  });
});
