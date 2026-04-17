//! DDR ファイルのインポートコマンド。

use std::path::Path;
use std::sync::Arc;

use crate::{
    commands::CommandError,
    db::repository::{
        get_project, get_solution_projects, insert_ddr_file, insert_solution, SolutionWithProjects,
    },
    parser::{normalize_link, parse_ddr, parse_summary},
    AppState,
};

// ---------------------------------------------------------------------------
// ビジネスロジック（テスト可能な部分を分離）
// ---------------------------------------------------------------------------

/// DDR ファイルのバイト列を UTF-8 文字列にデコードする。
///
/// FileMaker DDR は UTF-16 LE BOM (`\xFF\xFE`) または UTF-8 で出力される。
/// UTF-16 BE BOM (`\xFE\xFF`) および UTF-8 BOM (`\xEF\xBB\xBF`) にも対応。
pub(crate) fn decode_ddr_bytes(bytes: &[u8]) -> Result<String, String> {
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        // UTF-16 LE
        let words: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&words)
            .or_else(|_| Ok::<String, String>(String::from_utf16_lossy(&words)))
    } else if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        // UTF-16 BE
        let words: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_be_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16(&words)
            .or_else(|_| Ok::<String, String>(String::from_utf16_lossy(&words)))
    } else if bytes.len() >= 3 && bytes[0] == 0xEF && bytes[1] == 0xBB && bytes[2] == 0xBF {
        // UTF-8 BOM
        String::from_utf8(bytes[3..].to_vec()).map_err(|e| format!("UTF-8 デコードエラー: {e}"))
    } else {
        // UTF-8 (BOM なし)
        String::from_utf8(bytes.to_vec()).map_err(|e| format!("UTF-8 デコードエラー: {e}"))
    }
}

// ---------------------------------------------------------------------------
// Tauri コマンド
// ---------------------------------------------------------------------------

/// 概要.xml（Summary XML）を起点として、全 DDR ファイルをインポートする。
///
/// `summary_path`: 概要.xml のローカル絶対パス（フロントエンドからダイアログで取得）
#[tauri::command]
pub async fn import_solution(
    state: tauri::State<'_, AppState>,
    summary_path: String,
) -> Result<SolutionWithProjects, CommandError> {
    // 1. 概要.xml を読み込んでデコード
    let summary_bytes = tokio::fs::read(&summary_path)
        .await
        .map_err(|e| CommandError::Io(format!("ファイル読み込みエラー: {e}")))?;
    let summary_xml = decode_ddr_bytes(&summary_bytes)
        .map_err(|e| CommandError::Parse(format!("エンコーディングエラー: {e}")))?;

    // 2. 概要.xml をパース
    let entries = parse_summary(&summary_xml)
        .map_err(|e| CommandError::Parse(format!("概要XMLパースエラー: {e}")))?;

    // 3. 親ディレクトリを取得
    let summary_path_obj = Path::new(&summary_path);
    let parent_dir = summary_path_obj.parent().ok_or_else(|| {
        CommandError::Internal("summary_path の親ディレクトリが取得できません".into())
    })?;

    // 4. solution 名を決定（最初のエントリの name、またはファイル名）
    let solution_name = if let Some(first) = entries.first() {
        first.name.clone()
    } else {
        summary_path_obj
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string()
    };

    // 5. DB に solution を作成
    let solution_id = {
        let mut db = state
            .db
            .lock()
            .map_err(|e| CommandError::Internal(e.to_string()))?;
        insert_solution(&mut db, &solution_name, Some(&summary_path))
            .map_err(|e| CommandError::Database(format!("DB挿入エラー: {e}")))?
    };

    // 6. 各エントリの詳細 XML を読み込んでインポート
    for entry in &entries {
        let file_name = normalize_link(&entry.link);
        let detail_path = parent_dir.join(file_name);
        let detail_path_str = detail_path
            .to_str()
            .ok_or_else(|| CommandError::Internal("ファイルパスの変換に失敗しました".into()))?
            .to_string();

        let bytes = tokio::fs::read(&detail_path)
            .await
            .map_err(|e| CommandError::Io(format!("ファイル読み込みエラー ({file_name}): {e}")))?;
        let xml = decode_ddr_bytes(&bytes).map_err(|e| {
            CommandError::Parse(format!("エンコーディングエラー ({file_name}): {e}"))
        })?;

        let ddr = parse_ddr(&xml)
            .map_err(|e| CommandError::Parse(format!("パースエラー ({file_name}): {e}")))?;
        let ddr_arc = Arc::new(ddr);

        let project_id = {
            let mut db = state
                .db
                .lock()
                .map_err(|e| CommandError::Internal(e.to_string()))?;
            insert_ddr_file(&mut db, &ddr_arc, solution_id, Some(&detail_path_str))
                .map_err(|e| CommandError::Database(format!("DB挿入エラー ({file_name}): {e}")))?
        };

        // キャッシュに保存
        {
            let mut cache = state
                .ddr_cache
                .write()
                .map_err(|e| CommandError::Internal(e.to_string()))?;
            cache.insert(project_id, Arc::clone(&ddr_arc));
        }
    }

    // 7. solution + projects を取得して返す
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    let solution =
        crate::db::repository::get_solution(&db, solution_id).map_err(CommandError::from)?;
    let projects = get_solution_projects(&db, solution_id).map_err(CommandError::from)?;

    Ok(SolutionWithProjects { solution, projects })
}

/// DDR XML ファイルをインポートする（後方互換のため維持）。
///
/// `file_path`: ローカルファイルの絶対パス（フロントエンドからダイアログで取得）
#[tauri::command]
pub async fn import_ddr(
    state: tauri::State<'_, AppState>,
    file_path: String,
) -> Result<crate::db::repository::ProjectRow, CommandError> {
    // 1. ファイルをバイト列で読み込み（UTF-16 対応のため）
    let bytes = tokio::fs::read(&file_path)
        .await
        .map_err(|e| CommandError::Io(format!("ファイル読み込みエラー: {e}")))?;
    let xml = decode_ddr_bytes(&bytes)
        .map_err(|e| CommandError::Parse(format!("エンコーディングエラー: {e}")))?;

    // 2. パース（ロック前に実行してロック時間を短縮）
    let ddr = parse_ddr(&xml).map_err(|e| CommandError::Parse(format!("パースエラー: {e}")))?;
    let ddr_arc = Arc::new(ddr);

    // 3. solution + project を DB に保存
    let (_solution_id, project_id) = {
        let mut db = state
            .db
            .lock()
            .map_err(|e| CommandError::Internal(e.to_string()))?;
        let sid = insert_solution(&mut db, &ddr_arc.file_name, Some(&file_path))
            .map_err(|e| CommandError::Database(format!("DB挿入エラー: {e}")))?;
        let pid = insert_ddr_file(&mut db, &ddr_arc, sid, Some(&file_path))
            .map_err(|e| CommandError::Database(format!("DB挿入エラー: {e}")))?;
        (sid, pid)
    };

    // 4. キャッシュに保存
    {
        let mut cache = state
            .ddr_cache
            .write()
            .map_err(|e| CommandError::Internal(e.to_string()))?;
        cache.insert(project_id, Arc::clone(&ddr_arc));
    }

    // 5. ProjectRow を返す
    let db = state
        .db
        .lock()
        .map_err(|e| CommandError::Internal(e.to_string()))?;
    get_project(&db, project_id).map_err(CommandError::from)
}

/// テキストファイルをパスに書き込む（CSV エクスポート等に使用）。
/// パスはフロントエンドのネイティブ保存ダイアログから取得する。
#[tauri::command]
pub async fn write_text_file(path: String, content: String) -> Result<(), CommandError> {
    std::fs::write(&path, content.as_bytes())
        .map_err(|e| CommandError::Io(format!("ファイル書き込みエラー: {e}")))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::repository::insert_solution;
    use crate::db::Database;

    const MINIMAL_XML: &str = include_str!("../../../tests/fixtures/minimal.xml");

    // テスト専用: DB に solution + project を直接挿入するヘルパー
    pub(crate) fn insert_test_solution(
        db: &mut Database,
        xml: &str,
        file_path: Option<&str>,
    ) -> Result<SolutionWithProjects, String> {
        let ddr = parse_ddr(xml).map_err(|e| format!("パースエラー: {e}"))?;
        let sid = insert_solution(db, &ddr.file_name, file_path)
            .map_err(|e| format!("solution挿入エラー: {e}"))?;
        let pid = insert_ddr_file(db, &ddr, sid, file_path)
            .map_err(|e| format!("project挿入エラー: {e}"))?;
        let solution = crate::db::repository::get_solution(db, sid).map_err(|e| e.to_string())?;
        let project = get_project(db, pid).map_err(|e| e.to_string())?;
        Ok(SolutionWithProjects {
            solution,
            projects: vec![project],
        })
    }

    #[test]
    fn import_solution_creates_solution_and_project() {
        // 実ファイルIOを使ったテスト
        let summary_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tests/fixtures/minimal_summary.xml"
        );
        let bytes = std::fs::read(summary_path).unwrap();
        let summary_xml = decode_ddr_bytes(&bytes).unwrap();
        let entries = parse_summary(&summary_xml).unwrap();

        let summary_path_obj = Path::new(summary_path);
        let parent_dir = summary_path_obj.parent().unwrap();

        let mut db = Database::open_in_memory().unwrap();
        let sid = insert_solution(&mut db, &entries[0].name, Some(summary_path)).unwrap();

        for entry in &entries {
            let file_name = normalize_link(&entry.link);
            let detail_path = parent_dir.join(file_name);
            let detail_bytes = std::fs::read(&detail_path).unwrap();
            let xml = decode_ddr_bytes(&detail_bytes).unwrap();
            let ddr = parse_ddr(&xml).unwrap();
            let detail_path_str = detail_path.to_str().unwrap().to_string();
            insert_ddr_file(&mut db, &ddr, sid, Some(&detail_path_str)).unwrap();
        }

        let solution = crate::db::repository::get_solution(&db, sid).unwrap();
        let projects = get_solution_projects(&db, sid).unwrap();

        assert_eq!(solution.name, "TestDB.fmp12");
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "TestDB");
    }

    #[test]
    fn import_solution_file_not_found_returns_error() {
        let bytes = b"dummy";
        // 存在しないパス → fs::read がエラー
        let result = std::fs::read("/nonexistent/path/概要.xml");
        assert!(result.is_err());
        // decode_ddr_bytes は invalid UTF-8 でもエラーを返さない（latin1扱い）ので
        // parse_summary でエラーになることを確認
        let xml_result = decode_ddr_bytes(bytes);
        assert!(xml_result.is_ok());
        let parse_result = parse_summary(&xml_result.unwrap());
        assert!(parse_result.is_err());
    }

    #[test]
    fn decode_ddr_bytes_utf8() {
        let xml = b"<?xml version=\"1.0\"?><root/>";
        let result = decode_ddr_bytes(xml).unwrap();
        assert!(result.contains("<root/>"));
    }

    #[test]
    fn decode_ddr_bytes_utf16_le_bom() {
        // UTF-16 LE BOM + "AB"
        let mut bytes: Vec<u8> = vec![0xFF, 0xFE];
        for ch in "AB".encode_utf16() {
            bytes.extend_from_slice(&ch.to_le_bytes());
        }
        let result = decode_ddr_bytes(&bytes).unwrap();
        assert_eq!(result, "AB");
    }

    #[test]
    fn decode_ddr_bytes_utf8_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"hello");
        let result = decode_ddr_bytes(&bytes).unwrap();
        assert_eq!(result, "hello");
    }

    #[test]
    fn decode_ddr_bytes_utf16_le_invalid_surrogate_falls_back_lossy() {
        // UTF-16 LE BOM + 孤立サロゲート 0xD800 + 'A'
        // 0xD800 はサロゲートペアの前半のみで不正な UTF-16
        let mut bytes: Vec<u8> = vec![0xFF, 0xFE];
        bytes.extend_from_slice(&0xD800_u16.to_le_bytes()); // 孤立サロゲート
        bytes.extend_from_slice(&('A' as u16).to_le_bytes());
        // 修正前: from_utf16() がエラーを返すため Err になる
        // 修正後: from_utf16_lossy() にフォールバックして Ok (U+FFFD + 'A') になる
        let result = decode_ddr_bytes(&bytes);
        assert!(
            result.is_ok(),
            "不正UTF-16でもlossyフォールバックでOkを返すこと"
        );
        let s = result.unwrap();
        assert!(s.contains('A'), "有効な文字はそのまま含まれること");
        assert!(
            s.contains('\u{FFFD}'),
            "不正サロゲートはU+FFDDに置換されること"
        );
    }

    #[test]
    fn decode_ddr_bytes_utf16_be_invalid_surrogate_falls_back_lossy() {
        // UTF-16 BE BOM + 孤立サロゲート 0xDC00 + 'B'
        let mut bytes: Vec<u8> = vec![0xFE, 0xFF];
        bytes.extend_from_slice(&0xDC00_u16.to_be_bytes()); // 孤立サロゲート（後半のみ）
        bytes.extend_from_slice(&('B' as u16).to_be_bytes());
        let result = decode_ddr_bytes(&bytes);
        assert!(
            result.is_ok(),
            "BE版でも不正UTF-16でlossyフォールバックすること"
        );
        let s = result.unwrap();
        assert!(s.contains('B'));
        assert!(s.contains('\u{FFFD}'));
    }

    #[test]
    fn import_same_path_twice_creates_two_solutions() {
        let mut db = Database::open_in_memory().unwrap();
        let path = Some("test/path/概要.xml");

        let sid1 = insert_solution(&mut db, "TestDB", path).unwrap();
        let sid2 = insert_solution(&mut db, "TestDB", path).unwrap();

        assert_ne!(
            sid1, sid2,
            "同一パスを2回インポートすると異なるIDが発行される"
        );

        let solutions = crate::db::repository::list_solutions(&db).unwrap();
        assert_eq!(solutions.len(), 2, "ソリューションが2件存在する");
    }

    #[test]
    fn insert_test_solution_helper_works() {
        let mut db = Database::open_in_memory().unwrap();
        let result = insert_test_solution(&mut db, MINIMAL_XML, None).unwrap();
        assert_eq!(result.solution.name, "TestDB");
        assert_eq!(result.projects.len(), 1);
        assert_eq!(result.projects[0].name, "TestDB");
    }
}
