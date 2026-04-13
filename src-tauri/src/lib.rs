pub mod analyzer;
pub mod commands;
pub mod db;
pub mod parser;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
};

use crate::{db::Database, parser::models::DdrFile};

// ---------------------------------------------------------------------------
// アプリケーション状態
// ---------------------------------------------------------------------------

/// Tauri アプリ全体で共有する状態。
pub struct AppState {
    pub db: Mutex<Database>,
    /// project_id → DdrFile のインメモリキャッシュ。
    pub ddr_cache: RwLock<HashMap<i64, Arc<DdrFile>>>,
}

// ---------------------------------------------------------------------------
// エントリポイント
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::{
                menu::{MenuBuilder, MenuItemBuilder, SubmenuBuilder},
                Manager as _,
            };

            // アプリデータディレクトリに DB ファイルを作成
            let db_path = app
                .path()
                .app_data_dir()
                .map(|p| {
                    let _ = std::fs::create_dir_all(&p);
                    p.join("fm_ddr.db")
                })
                .unwrap_or_else(|_| std::path::PathBuf::from("fm_ddr.db"));

            let db = Database::open(db_path.to_str().unwrap_or("fm_ddr.db"))?;
            app.manage(AppState {
                db: Mutex::new(db),
                ddr_cache: RwLock::new(HashMap::new()),
            });

            // --------------- メニューバー構築 ---------------
            let quit = MenuItemBuilder::with_id("quit", "終了")
                .accelerator("CmdOrControl+Q")
                .build(app)?;

            let file_menu = SubmenuBuilder::new(app, "ファイル").item(&quit).build()?;

            let font_increase = MenuItemBuilder::with_id("font-increase", "拡大")
                .accelerator("CmdOrControl+Equal")
                .build(app)?;
            let font_decrease = MenuItemBuilder::with_id("font-decrease", "縮小")
                .accelerator("CmdOrControl+Minus")
                .build(app)?;
            let font_reset = MenuItemBuilder::with_id("font-reset", "標準サイズに戻す")
                .accelerator("CmdOrControl+0")
                .build(app)?;

            let view_menu = SubmenuBuilder::new(app, "表示")
                .item(&font_increase)
                .item(&font_decrease)
                .item(&font_reset)
                .build()?;

            let open_upgrade_settings =
                MenuItemBuilder::with_id("open-upgrade-settings", "アップグレードチェック設定...")
                    .build(app)?;
            let edit_menu = SubmenuBuilder::new(app, "編集")
                .item(&open_upgrade_settings)
                .build()?;

            let about = MenuItemBuilder::with_id("about", "バージョン情報").build(app)?;

            let help_menu = SubmenuBuilder::new(app, "ヘルプ").item(&about).build()?;

            let menu = MenuBuilder::new(app)
                .item(&file_menu)
                .item(&edit_menu)
                .item(&view_menu)
                .item(&help_menu)
                .build()?;

            app.set_menu(menu)?;

            // --------------- メニューイベント ---------------
            app.on_menu_event(|app, event| {
                use tauri::Emitter as _;
                match event.id().as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "font-increase" => {
                        let _ = app.emit("font-size-step", 1i32);
                    }
                    "font-decrease" => {
                        let _ = app.emit("font-size-step", -1i32);
                    }
                    "font-reset" => {
                        let _ = app.emit("font-size-step", 0i32);
                    }
                    "about" => {
                        let _ = app.emit("show-about", ());
                    }
                    "open-upgrade-settings" => {
                        let _ = app.emit("open-upgrade-settings", ());
                    }
                    _ => {}
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::import::import_solution,
            commands::import::import_ddr,
            commands::import::write_text_file,
            commands::search::search_elements,
            commands::analysis::list_solutions,
            commands::analysis::get_solution_projects,
            commands::analysis::delete_solution,
            commands::analysis::list_projects,
            commands::analysis::delete_project,
            commands::analysis::get_project_summary,
            commands::analysis::get_broken_refs,
            commands::analysis::get_report_card,
            commands::analysis::resolve_element_by_name,
            commands::catalog::list_all_fields,
            commands::catalog::list_tables,
            commands::catalog::list_table_fields,
            commands::catalog::list_scripts,
            commands::catalog::list_script_steps,
            commands::catalog::list_layouts,
            commands::catalog::list_layout_triggers,
            commands::catalog::list_layout_objects,
            commands::catalog::list_layout_object_conditions,
            commands::catalog::list_value_lists,
            commands::catalog::list_value_list_items,
            commands::catalog::list_custom_functions,
            commands::catalog::list_table_occurrences,
            commands::catalog::list_relationships,
            commands::catalog::list_accounts,
            commands::catalog::list_privilege_sets,
            commands::field_refs::resolve_layout_field,
            commands::field_refs::get_field_refs,
            commands::field_refs::get_field_calc_refs,
            commands::field_refs::get_field_layout_refs,
            commands::field_refs::get_layout_ref_debug_info,
            commands::field_refs::get_field_relationship_keys,
            commands::field_refs::list_unused_fields,
            commands::callchain::get_call_chain,
            commands::callchain::get_callers,
            commands::callchain::get_orphan_scripts,
            commands::diff::compare_projects,
            commands::diff::compare_solutions,
            commands::diff::list_all_projects,
            commands::analysis::get_upgrade_check,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
