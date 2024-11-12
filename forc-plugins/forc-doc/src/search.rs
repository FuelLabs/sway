use crate::doc::{module::ModuleInfo, Document, Documentation};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::Path,
};

const JS_SEARCH_FILE_NAME: &str = "search.js";

/// Creates the search index javascript file for the search bar.
pub fn write_search_index(doc_path: &Path, docs: &Documentation) -> Result<()> {
    let json_data = docs.to_search_index_json_value()?;
    let module_export =
        "\"object\"==typeof exports&&\"undefined\"!=typeof module&&(module.exports=SEARCH_INDEX);";
    let js_data = format!("var SEARCH_INDEX={json_data};\n{module_export}");
    Ok(fs::write(doc_path.join(JS_SEARCH_FILE_NAME), js_data)?)
}

impl Documentation {
    /// Generates a mapping of program name to a vector of documentable items within the program
    /// and returns the map as a `serde_json::Value`.
    fn to_search_index_json_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        let mut map = HashMap::with_capacity(self.len());
        let mut modules = BTreeMap::new();
        for doc in self.iter() {
            let project_name = doc.module_info.project_name().to_string();
            map.entry(project_name)
                .or_insert_with(Vec::new)
                .push(JsonSearchItem::from(doc));
            modules.insert(
                doc.module_info.module_prefixes.join("::"),
                doc.module_info.clone(),
            );
        }

        // Insert the modules themselves into the map.
        for (_, module) in modules.iter() {
            let project_name = module.project_name().to_string();
            map.entry(project_name)
                .or_insert_with(Vec::new)
                .push(JsonSearchItem::from(module));
        }

        serde_json::to_value(map)
    }
}

/// Item information used in the `search_pool.json`.
/// The item name is what the fuzzy search will be
/// matching on, all other information will be used
/// in generating links to the item.
#[derive(Clone, Debug, Serialize, Deserialize)]
struct JsonSearchItem {
    name: String,
    html_filename: String,
    module_info: Vec<String>,
    preview: String,
    type_name: String,
}
impl<'a> From<&'a Document> for JsonSearchItem {
    fn from(value: &'a Document) -> Self {
        Self {
            name: value.item_body.item_name.to_string(),
            html_filename: value.html_filename(),
            module_info: value.module_info.module_prefixes.clone(),
            preview: value
                .preview_opt()
                .unwrap_or_default()
                .replace("<br>", "")
                .replace("<p>", "")
                .replace("</p>", ""),
            type_name: value.item_body.ty.friendly_type_name().into(),
        }
    }
}

impl<'a> From<&'a ModuleInfo> for JsonSearchItem {
    fn from(value: &'a ModuleInfo) -> Self {
        Self {
            name: value
                .module_prefixes
                .last()
                .unwrap_or(&String::new())
                .to_string(),
            html_filename: "index.html".into(),
            module_info: value.module_prefixes.clone(),
            preview: value
                .preview_opt()
                .unwrap_or_default()
                .replace("<br>", "")
                .replace("<p>", "")
                .replace("</p>", ""),
            type_name: "module".into(),
        }
    }
}
