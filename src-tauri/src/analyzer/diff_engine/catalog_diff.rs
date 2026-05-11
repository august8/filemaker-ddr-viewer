use std::collections::HashMap;

use crate::parser::models::{JoinPredicate, Relationship, TableOccurrence};

use super::{DiffItem, DiffKind};

/// テーブルオカレンス専用の差分検出。
pub(super) fn diff_table_occurrences(
    old_tos: &[TableOccurrence],
    new_tos: &[TableOccurrence],
    out: &mut Vec<DiffItem>,
) {
    let old_map: HashMap<&str, &TableOccurrence> = old_tos
        .iter()
        .map(|t| (t.occurrence_name.as_str(), t))
        .collect();
    let new_map: HashMap<&str, &TableOccurrence> = new_tos
        .iter()
        .map(|t| (t.occurrence_name.as_str(), t))
        .collect();

    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Added,
                element_type: "table_occurrence".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Removed,
                element_type: "table_occurrence".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    for (name, old_to) in &old_map {
        if let Some(new_to) = new_map.get(name) {
            if old_to.base_table_name != new_to.base_table_name {
                out.push(DiffItem {
                    kind: DiffKind::Modified,
                    element_type: "table_occurrence".into(),
                    name: (*name).to_owned(),
                    detail: Some(format!(
                        "ベーステーブル: {} → {}",
                        old_to.base_table_name, new_to.base_table_name
                    )),
                    project_id: None,
                    compare_project_id: None,
                });
            }
        }
    }
}

/// リレーション専用の差分検出。
pub(super) fn diff_relationships(
    old_rels: &[Relationship],
    new_rels: &[Relationship],
    out: &mut Vec<DiffItem>,
) {
    let old_map: HashMap<&str, &Relationship> =
        old_rels.iter().map(|r| (r.name.as_str(), r)).collect();
    let new_map: HashMap<&str, &Relationship> =
        new_rels.iter().map(|r| (r.name.as_str(), r)).collect();

    for name in new_map.keys() {
        if !old_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Added,
                element_type: "relationship".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    for name in old_map.keys() {
        if !new_map.contains_key(name) {
            out.push(DiffItem {
                kind: DiffKind::Removed,
                element_type: "relationship".into(),
                name: (*name).to_owned(),
                detail: None,
                project_id: None,
                compare_project_id: None,
            });
        }
    }

    for (name, old_rel) in &old_map {
        if let Some(new_rel) = new_map.get(name) {
            let old_sig = relationship_signature(old_rel);
            let new_sig = relationship_signature(new_rel);
            if old_sig != new_sig {
                out.push(DiffItem {
                    kind: DiffKind::Modified,
                    element_type: "relationship".into(),
                    name: (*name).to_owned(),
                    detail: Some(format!("{old_sig} → {new_sig}")),
                    project_id: None,
                    compare_project_id: None,
                });
            }
        }
    }
}

/// リレーションの変化検出用シグネチャ（テーブル・predicates を含む）。
fn relationship_signature(rel: &Relationship) -> String {
    let preds: String = rel
        .predicates
        .iter()
        .map(predicate_signature)
        .collect::<Vec<_>>()
        .join(";");
    format!("{}={}|{}", rel.left_table, rel.right_table, preds)
}

fn predicate_signature(p: &JoinPredicate) -> String {
    format!("{} {} {}", p.left_field, p.operator, p.right_field)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::super::{diff_ddr, DiffKind};
    use crate::parser::models::*;
    use crate::parser::version::FmVersion;

    fn make_base() -> DdrFile {
        DdrFile {
            file_name: "A".into(),
            fm_version: FmVersion {
                major: 21,
                minor: 0,
                patch: "v1".into(),
            },
            tables: vec![],
            scripts: vec![],
            layouts: vec![],
            relationships: vec![],
            value_lists: vec![],
            custom_functions: vec![],
            accounts: vec![],
            privilege_sets: vec![],
            table_occurrences: vec![],
            file_script_triggers: vec![],
            external_data_sources: vec![],
        }
    }

    fn make_to(occurrence_name: &str, base_table_name: &str) -> TableOccurrence {
        TableOccurrence {
            occurrence_name: occurrence_name.into(),
            base_table_name: base_table_name.into(),
            source_file: None,
        }
    }

    fn make_pred(left: &str, right: &str) -> JoinPredicate {
        JoinPredicate {
            left_field: left.into(),
            right_field: right.into(),
            operator: "Equal".into(),
        }
    }

    fn make_rel(name: &str, left: &str, right: &str, preds: Vec<JoinPredicate>) -> Relationship {
        Relationship {
            id: RelationshipId(1),
            name: name.into(),
            left_table: left.into(),
            right_table: right.into(),
            predicates: preds,
        }
    }

    #[test]
    fn diff_table_occurrences_detects_added() {
        let mut base = make_base();
        base.table_occurrences = vec![make_to("Contact", "Contact")];
        let mut updated = base.clone();
        updated.table_occurrences =
            vec![make_to("Contact", "Contact"), make_to("Invoice", "Invoice")];
        let result = diff_ddr(&base, &updated);
        let to_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "table_occurrence")
            .collect();
        assert_eq!(to_items.len(), 1);
        assert_eq!(to_items[0].kind, DiffKind::Added);
        assert_eq!(to_items[0].name, "Invoice");
    }

    #[test]
    fn diff_table_occurrences_detects_removed() {
        let mut base = make_base();
        base.table_occurrences = vec![make_to("Contact", "Contact"), make_to("OldTO", "OldTable")];
        let mut updated = base.clone();
        updated.table_occurrences = vec![make_to("Contact", "Contact")];
        let result = diff_ddr(&base, &updated);
        let to_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "table_occurrence")
            .collect();
        assert_eq!(to_items.len(), 1);
        assert_eq!(to_items[0].kind, DiffKind::Removed);
        assert_eq!(to_items[0].name, "OldTO");
    }

    #[test]
    fn diff_table_occurrences_detects_base_table_change() {
        let mut base = make_base();
        base.table_occurrences = vec![make_to("Contact", "OldTable")];
        let mut updated = base.clone();
        updated.table_occurrences = vec![make_to("Contact", "NewTable")];
        let result = diff_ddr(&base, &updated);
        let to_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "table_occurrence")
            .collect();
        assert_eq!(to_items.len(), 1);
        assert_eq!(to_items[0].kind, DiffKind::Modified);
        assert_eq!(to_items[0].name, "Contact");
        let detail = to_items[0].detail.as_deref().unwrap_or("");
        assert!(
            detail.contains("OldTable"),
            "detail should contain old value: {detail}"
        );
        assert!(
            detail.contains("NewTable"),
            "detail should contain new value: {detail}"
        );
    }

    #[test]
    fn diff_table_occurrences_no_change_is_skipped() {
        let mut base = make_base();
        base.table_occurrences = vec![make_to("Contact", "Contact")];
        let mut updated = base.clone();
        updated.table_occurrences = vec![make_to("Contact", "Contact")];
        let result = diff_ddr(&base, &updated);
        let to_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "table_occurrence")
            .collect();
        assert!(
            to_items.is_empty(),
            "identical TO should not appear in diff"
        );
    }

    #[test]
    fn diff_relationships_detects_added() {
        let mut base = make_base();
        base.relationships = vec![make_rel(
            "Rel1",
            "Contact",
            "Invoice",
            vec![make_pred("id", "contact_id")],
        )];
        let mut updated = base.clone();
        updated
            .relationships
            .push(make_rel("Rel2", "Invoice", "LineItem", vec![]));
        let result = diff_ddr(&base, &updated);
        let rel_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "relationship")
            .collect();
        assert_eq!(rel_items.len(), 1);
        assert_eq!(rel_items[0].kind, DiffKind::Added);
        assert_eq!(rel_items[0].name, "Rel2");
    }

    #[test]
    fn diff_relationships_detects_removed() {
        let mut base = make_base();
        base.relationships = vec![
            make_rel("Rel1", "Contact", "Invoice", vec![]),
            make_rel("OldRel", "A", "B", vec![]),
        ];
        let mut updated = base.clone();
        updated.relationships = vec![make_rel("Rel1", "Contact", "Invoice", vec![])];
        let result = diff_ddr(&base, &updated);
        let rel_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "relationship")
            .collect();
        assert_eq!(rel_items.len(), 1);
        assert_eq!(rel_items[0].kind, DiffKind::Removed);
        assert_eq!(rel_items[0].name, "OldRel");
    }

    #[test]
    fn diff_relationships_detects_predicate_change() {
        let mut base = make_base();
        base.relationships = vec![make_rel(
            "Rel1",
            "Contact",
            "Invoice",
            vec![make_pred("id", "contact_id")],
        )];
        let mut updated = base.clone();
        updated.relationships = vec![make_rel(
            "Rel1",
            "Contact",
            "Invoice",
            vec![make_pred("id", "cid")],
        )];
        let result = diff_ddr(&base, &updated);
        let rel_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "relationship")
            .collect();
        assert_eq!(rel_items.len(), 1);
        assert_eq!(rel_items[0].kind, DiffKind::Modified);
    }

    #[test]
    fn diff_relationships_no_change_is_skipped() {
        let pred = make_pred("id", "contact_id");
        let mut base = make_base();
        base.relationships = vec![make_rel("Rel1", "Contact", "Invoice", vec![pred.clone()])];
        let mut updated = base.clone();
        updated.relationships = vec![make_rel("Rel1", "Contact", "Invoice", vec![pred])];
        let result = diff_ddr(&base, &updated);
        let rel_items: Vec<_> = result
            .items
            .iter()
            .filter(|i| i.element_type == "relationship")
            .collect();
        assert!(
            rel_items.is_empty(),
            "identical relationship should not appear in diff"
        );
    }
}
