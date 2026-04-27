//! 実 FileMaker DDR ファイルを使ったバージョン互換統合テスト。
//!
//! `tests/ddr/<version>/BaseFile_fmp12.xml` を全バージョン分パースし、
//! パーサーが FM17〜22 の実出力に対して正常に動作することを検証する。
//! テーブルなしファイルのパースも検証する。

use filemaker_ddr_viewer_lib::analyzer::broken_refs::{find_broken_refs, BrokenRefKind};
use filemaker_ddr_viewer_lib::parser::{decode_ddr_bytes, parse_ddr};
use rstest::rstest;

#[rstest]
#[case("17.0.7.700")]
#[case("18.0.3.317")]
#[case("19.6.3.302")]
#[case("20.3.2.201")]
#[case("21.1.2.200")]
#[case("22.0.6.601")]
fn parse_real_ddr_succeeds(#[case] version: &str) {
    let path = format!("../tests/ddr/{version}/BaseFile_fmp12.xml");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let xml = decode_ddr_bytes(&bytes).expect("decode failed");
    let result = parse_ddr(&xml);
    assert!(
        result.is_ok(),
        "FM{version} のパースが失敗: {:?}",
        result.err()
    );
}

#[rstest]
#[case("17.0.7.700", 17)]
#[case("18.0.3.317", 18)]
#[case("19.6.3.302", 19)]
#[case("20.3.2.201", 20)]
#[case("21.1.2.200", 21)]
#[case("22.0.6.601", 22)]
fn parse_real_ddr_version_detected(#[case] version: &str, #[case] expected_major: u32) {
    let path = format!("../tests/ddr/{version}/BaseFile_fmp12.xml");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let xml = decode_ddr_bytes(&bytes).expect("decode failed");
    let ddr = parse_ddr(&xml).unwrap_or_else(|e| panic!("FM{version} パース失敗: {e}"));
    assert_eq!(
        ddr.fm_version.major, expected_major,
        "FM{version}: メジャーバージョンが一致しない"
    );
}

#[rstest]
#[case("17.0.7.700")]
#[case("18.0.3.317")]
#[case("19.6.3.302")]
#[case("20.3.2.201")]
#[case("21.1.2.200")]
#[case("22.0.6.601")]
fn parse_real_ddr_has_tables_and_fields(#[case] version: &str) {
    let path = format!("../tests/ddr/{version}/BaseFile_fmp12.xml");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let xml = decode_ddr_bytes(&bytes).expect("decode failed");
    let ddr = parse_ddr(&xml).unwrap_or_else(|e| panic!("FM{version} パース失敗: {e}"));
    assert!(!ddr.tables.is_empty(), "FM{version}: テーブルが0件");
    let total_fields: usize = ddr.tables.iter().map(|t| t.fields.len()).sum();
    assert!(total_fields > 0, "FM{version}: フィールドが0件");
}

/// FM22 の DDR には壊れたフィールド参照・レイアウト参照が含まれており、
/// find_broken_refs が BrokenFieldRef / BrokenLayoutRef を返すことを確認する。
#[test]
fn fm22_broken_step_refs_detected() {
    let path = "../tests/ddr/22.0.6.601/BaseFile_fmp12.xml";
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let xml = decode_ddr_bytes(&bytes).expect("decode failed");
    let ddr = parse_ddr(&xml).unwrap_or_else(|e| panic!("FM22 パース失敗: {e}"));

    let refs = find_broken_refs(&ddr);
    assert!(
        refs.iter().any(|r| r.kind == BrokenRefKind::BrokenFieldRef),
        "BrokenFieldRef が検出されなかった: {refs:?}"
    );
    assert!(
        refs.iter()
            .any(|r| r.kind == BrokenRefKind::BrokenLayoutRef),
        "BrokenLayoutRef が検出されなかった: {refs:?}"
    );
}

#[test]
fn parse_no_table_ddr_succeeds() {
    let path = "../tests/ddr/NoTableDDR/NoTableFile_fmp12.xml";
    let bytes = std::fs::read(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let xml = decode_ddr_bytes(&bytes).expect("decode failed");
    let ddr = parse_ddr(&xml).unwrap_or_else(|e| panic!("テーブルなし DDR のパース失敗: {e}"));
    assert!(
        ddr.tables.is_empty(),
        "テーブルなしファイルなのにテーブルが検出された"
    );
}
