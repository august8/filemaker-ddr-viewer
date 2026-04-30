// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // test-utils ビルドでは WebView2 に CDP ポートを渡す（Playwright E2E テスト用）
    // WebView2 初期化より前に設定する必要があるため main() の先頭で行う
    #[cfg(feature = "test-utils")]
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--remote-debugging-port=9222",
    );
    filemaker_ddr_viewer_lib::run()
}
