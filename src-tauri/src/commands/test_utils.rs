//! テスト専用 IPC コマンド。`--features test-utils` でのみコンパイルされる。
//! リリースバイナリには含まれない。

use crate::{commands::CommandError, db::repository::SolutionWithProjects, AppState};

#[tauri::command]
pub async fn import_ddr_from_path(
    state: tauri::State<'_, AppState>,
    summary_path: String,
) -> Result<SolutionWithProjects, CommandError> {
    super::import::import_solution(state, summary_path).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_minimal_summary_exists() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/fixtures/minimal_summary.xml"
        );
        assert!(
            std::fs::metadata(path).is_ok(),
            "E2E テスト用フィクスチャ minimal_summary.xml が存在する"
        );
    }

    #[test]
    fn fixture_fm22_ddr_exists() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/ddr/22.0.6.601/概要.xml"
        );
        assert!(
            std::fs::metadata(path).is_ok(),
            "E2E ゴールデンパス用 FM22 DDR 概要.xml が存在する"
        );
    }
}
