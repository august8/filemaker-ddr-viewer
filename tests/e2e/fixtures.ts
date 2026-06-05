import { test as base, chromium, type Page } from "@playwright/test";

const CDP_URL = "http://localhost:9222";

export const test = base.extend<{ page: Page }>({
  // eslint-disable-next-line no-empty-pattern
  async page({}, use) {
    const browser = await chromium.connectOverCDP(CDP_URL);
    const context = browser.contexts()[0];
    // CDP 応答後も WebView2 がページを開くまでわずかな遅延があるため、
    // pages() が空の場合はページ生成イベントを待つ
    const page = context.pages()[0]
      ?? await context.waitForEvent("page", { timeout: 10_000 });
    await use(page);
    // browser.close() は呼ばない（globalTeardown でプロセスごと終了するため）
  },
});

export { expect } from "@playwright/test";
