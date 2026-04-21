mod field_extractor;
mod layout_catalog;
mod object_scanner;
mod script_and_format;

use crate::parser::models::{LayoutFieldRef, LayoutObject, ScriptTrigger};

pub use layout_catalog::parse_layouts;

/// `scan_object` / `scan_object_list` の戻り値型エイリアス。
pub(super) type ObjectScanResult = (
    Vec<ScriptTrigger>,
    Vec<String>,
    Vec<LayoutFieldRef>,
    Vec<LayoutObject>,
);
