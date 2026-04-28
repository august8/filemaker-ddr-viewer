import { fileURLToPath } from "url";
import path from "path";

const __dirname = fileURLToPath(new URL(".", import.meta.url));
const DDR_SUMMARY = path.resolve(__dirname, "../ddr/22.0.6.601/概要.xml");

type TauriWindow = Window & {
  __TAURI__: { core: { invoke: (cmd: string, args: unknown) => Promise<unknown> } };
};

describe("Golden path: DDR import → 検索 → 詳細表示", () => {
  it("1. import_ddr_from_path でソリューションをインポートできる", async () => {
    const result = await browser.execute(
      async (summaryPath: string) =>
        (window as TauriWindow).__TAURI__.core.invoke("import_ddr_from_path", { summaryPath }),
      DDR_SUMMARY
    );

    expect(result).toBeDefined();
    const res = result as { solution: unknown; projects: unknown[] };
    expect(res.solution).toBeTruthy();
    expect(res.projects.length).toBeGreaterThan(0);
  });

  it("2. サイドバーにソリューション名が表示される", async () => {
    const el = await $("*=BaseFile.fmp12");
    await el.waitForDisplayed({ timeout: 10_000 });
    await expect(el).toBeDisplayed();
  });

  it("3. 検索バーでキーワード検索できる", async () => {
    const searchInput = await $('[placeholder*="検索"]');
    await searchInput.setValue("BaseFile");
    // FTS5 検索は 300ms デバウンス後に実行されるため 400ms 待機
    await browser.pause(400);

    const results = await $$('[data-testid="search-result-item"]');
    expect(results.length).toBeGreaterThan(0);
  });

  it("4. 検索結果クリックで詳細パネルが表示される", async () => {
    const firstResult = await $('[data-testid="search-result-item"]');
    await firstResult.click();

    const mainContent = await $("main");
    await mainContent.waitForDisplayed({ timeout: 10_000 });
    await expect(mainContent).toBeDisplayed();
  });
});
