//! スクリプト呼び出しチェーン解析コマンド。

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    analyzer::call_chain::{build_call_chain, find_callers, CallChainNode},
    commands::CommandError,
    parser::models::{DdrFile, ScriptId},
    AppState,
};

// ---------------------------------------------------------------------------
// 公開型（フロントエンド向けに再エクスポート）
// ---------------------------------------------------------------------------

pub use crate::analyzer::call_chain::CallChainNode as CallChainNodeExport;

// ---------------------------------------------------------------------------
// 内部ヘルパー: キャッシュまたは DB からDdrFile を取得
// ---------------------------------------------------------------------------

pub(crate) fn get_ddr(
    state: &tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<Arc<DdrFile>, CommandError> {
    // キャッシュから取得を試みる
    {
        let cache = state
            .ddr_cache
            .read()
            .map_err(|e| CommandError::Internal(e.to_string()))?;
        if let Some(ddr) = cache.get(&project_id) {
            return Ok(Arc::clone(ddr));
        }
    }

    // キャッシュにない場合はファイルパスから再読み込み
    let file_path = {
        let db = state
            .db
            .lock()
            .map_err(|e| CommandError::Internal(e.to_string()))?;
        crate::db::repository::get_project(&db, project_id)
            .map_err(CommandError::from)?
            .file_path
    };

    let path = file_path.ok_or_else(|| {
        CommandError::NotFound("file_path が記録されていないため再読み込みできません".to_string())
    })?;

    let bytes = std::fs::read(&path).map_err(CommandError::from)?;
    let xml = crate::commands::import::decode_ddr_bytes(&bytes)
        .map_err(|e| CommandError::Parse(format!("デコードエラー: {e}")))?;

    let ddr = crate::parser::parse_ddr(&xml)
        .map_err(|e| CommandError::Parse(format!("パースエラー: {e}")))?;

    let ddr_arc = Arc::new(ddr);

    // キャッシュに保存
    let mut cache = state
        .ddr_cache
        .write()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    cache.insert(project_id, Arc::clone(&ddr_arc));

    Ok(ddr_arc)
}

// ---------------------------------------------------------------------------
// Tauri コマンド
// ---------------------------------------------------------------------------

/// スクリプトの呼び出しチェーンを返す。
#[tauri::command]
pub async fn get_call_chain(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    script_id: i64,
) -> Result<CallChainNode, CommandError> {
    let ddr = get_ddr(&state, project_id)?;
    build_call_chain(&ddr, ScriptId(script_id as u64), None)
        .map_err(|e| CommandError::Internal(e.to_string()))
}

/// 指定スクリプトを呼び出している呼び出し元スクリプト ID 一覧を返す。
#[tauri::command]
pub async fn get_callers(
    state: tauri::State<'_, AppState>,
    project_id: i64,
    script_id: i64,
) -> Result<Vec<u64>, CommandError> {
    let ddr = get_ddr(&state, project_id)?;
    let callers = find_callers(&ddr, ScriptId(script_id as u64));
    Ok(callers.iter().map(|id| id.0).collect())
}

// ---------------------------------------------------------------------------
// 未使用スクリプト（孤立スクリプト）コマンド
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrphanScriptDto {
    pub script_id: u64,
    pub script_name: String,
}

/// 未使用スクリプト（どこからも呼ばれていないスクリプト）一覧を返す。
#[tauri::command]
pub async fn get_orphan_scripts(
    state: tauri::State<'_, AppState>,
    project_id: i64,
) -> Result<Vec<OrphanScriptDto>, CommandError> {
    let ddr = get_ddr(&state, project_id)?;
    let orphans = crate::analyzer::orphans::find_orphan_scripts(&ddr);
    Ok(orphans
        .into_iter()
        .map(|o| OrphanScriptDto {
            script_id: o.script_id,
            script_name: o.script_name,
        })
        .collect())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL_XML: &str = include_str!("../../../tests/fixtures/minimal.xml");

    fn setup_ddr() -> Arc<DdrFile> {
        Arc::new(crate::parser::parse_ddr(MINIMAL_XML).unwrap())
    }

    #[test]
    fn call_chain_root_has_correct_depth() {
        let ddr = setup_ddr();
        let Some(script) = ddr.scripts.first() else {
            return;
        };
        let node = build_call_chain(&ddr, script.id, None).unwrap();
        assert_eq!(node.depth, 0);
        assert_eq!(node.script_id, script.id.0);
    }

    #[test]
    fn unknown_script_id_returns_error() {
        let ddr = setup_ddr();
        let result = build_call_chain(&ddr, ScriptId(99999), None);
        assert!(result.is_err());
    }

    #[test]
    fn orphan_scripts_count_is_lte_total() {
        let ddr = setup_ddr();
        let orphans = crate::analyzer::orphans::find_orphan_scripts(&ddr);
        assert!(orphans.len() <= ddr.scripts.len());
    }
}
