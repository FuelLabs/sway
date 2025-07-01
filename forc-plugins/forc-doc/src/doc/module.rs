//! Handles the gathering of module information used in navigation and documentation of modules.
use crate::render::{util::format::docstring::create_preview, INDEX_FILENAME};
use anyhow::Result;
use horrorshow::{box_html, Template};
use std::{fmt::Write, path::PathBuf};
use sway_core::language::CallPath;

pub(crate) type ModulePrefixes = Vec<String>;

/// Information about a Sway module.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct ModuleInfo {
    /// The preceding module names, used in navigating between modules.
    pub module_prefixes: ModulePrefixes,
    /// Doc attributes of a module.
    /// Renders into the module level docstrings.
    ///
    /// ```sway
    /// //! Module level docstring
    /// library;
    /// ```
    pub(crate) attributes: Option<String>,
}
impl ModuleInfo {
    /// The current module.
    ///
    /// Panics if there are no modules.
    pub(crate) fn location(&self) -> &str {
        self.module_prefixes
            .last()
            .expect("Expected Some module location, found None")
    }
    /// The name of the project.
    ///
    /// Panics if the project root is missing.
    pub(crate) fn project_name(&self) -> &str {
        self.module_prefixes
            .first()
            .expect("Expected root module, project root missing")
    }
    /// The location of the parent of the current module.
    ///
    /// Returns `None` if there is no parent.
    pub(crate) fn parent(&self) -> Option<&String> {
        if self.has_parent() {
            let mut iter = self.module_prefixes.iter();
            iter.next_back();
            iter.next_back()
        } else {
            None
        }
    }
    /// Determines if the current module has a parent module.
    fn has_parent(&self) -> bool {
        self.depth() > 1
    }
    pub(crate) fn is_root_module(&self) -> bool {
        self.location() == self.project_name()
    }
    /// Create a qualified path literal String that represents the full path to an item.
    ///
    /// Example: `project_name::module::Item`
    pub(crate) fn to_path_literal_string(&self, item_name: &str, location: &str) -> String {
        let prefix = self.to_path_literal_prefix(location);
        match prefix.is_empty() {
            true => item_name.to_owned(),
            false => format!("{prefix}::{item_name}"),
        }
    }
    /// Create a path literal prefix from the module prefixes.
    /// Use in `to_path_literal_string()` to create a full literal path string.
    ///
    /// Example: `module::submodule`
    fn to_path_literal_prefix(&self, location: &str) -> String {
        let mut iter = self.module_prefixes.iter();
        for prefix in iter.by_ref() {
            if prefix == location {
                break;
            }
        }
        iter.map(String::as_str).collect::<Vec<&str>>().join("::")
    }
    /// Renders the [ModuleInfo] into a [CallPath] with anchors. We return this as a `Result<Vec<String>>`
    /// since the `box_html!` macro returns a closure and no two closures are considered the same type.
    pub(crate) fn get_anchors(&self) -> Result<Vec<String>> {
        let mut count = self.depth();
        let mut rendered_module_anchors = Vec::with_capacity(self.depth());
        for prefix in &self.module_prefixes {
            let mut href = (1..count).map(|_| "../").collect::<String>();
            href.push_str(INDEX_FILENAME);
            rendered_module_anchors.push(
                box_html! {
                    a(class="mod", href=href) {
                        : prefix;
                    }
                    span: "::";
                }
                .into_string()?,
            );
            count -= 1;
        }
        Ok(rendered_module_anchors)
    }
    /// Creates a String version of the path to an item,
    /// used in navigation between pages. The location given is the break point.
    ///
    /// This is only used for full path syntax, e.g `module/submodule/file_name.html`.
    pub(crate) fn file_path_at_location(&self, file_name: &str, location: &str) -> Result<String> {
        let mut iter = self.module_prefixes.iter();
        for prefix in iter.by_ref() {
            if prefix == location {
                break;
            }
        }
        let mut file_path = iter.collect::<PathBuf>();
        file_path.push(file_name);

        file_path
            .to_str()
            .map(|file_path_str| file_path_str.to_string())
            .ok_or_else(|| anyhow::anyhow!("There will always be at least the item name"))
    }

    /// Compares the current `module_info` to the next `module_info` to determine how many directories to go back to make
    /// the next file path valid, and returns that path as a `String`.
    ///
    /// Example:
    /// ```
    /// // number of dirs:               [match][    2    ][    1    ]
    /// let current_location = "project_root/module/submodule1/submodule2/struct.Name.html";
    /// let next_location    =              "module/other_submodule/enum.Name.html";
    /// let result           =               "../../other_submodule/enum.Name.html";
    /// ```
    /// In this case the first module to match is "module", so we have no need to go back further than that.
    pub(crate) fn file_path_from_location(
        &self,
        file_name: &str,
        current_module_info: &ModuleInfo,
        is_external_item: bool,
    ) -> Result<String> {
        if is_external_item {
            let mut new_path = (0..current_module_info.module_prefixes.len())
                .map(|_| "../")
                .collect::<String>();
            write!(new_path, "{}/{}", self.module_prefixes.join("/"), file_name)?;
            Ok(new_path)
        } else {
            let mut mid = 0; // the index to split the module_info from call_path at
            let mut offset = 0; // the number of directories to go back
            let mut next_location_iter = self.module_prefixes.iter().rev().enumerate().peekable();
            while let Some((index, prefix)) = next_location_iter.peek() {
                for (count, module) in current_module_info.module_prefixes.iter().rev().enumerate()
                {
                    if module == *prefix {
                        offset = count;
                        mid = self.module_prefixes.len() - index;
                        break;
                    }
                }
                next_location_iter.next();
            }
            let mut new_path = (0..offset).map(|_| "../").collect::<String>();
            write!(
                new_path,
                "{}/{}",
                self.module_prefixes.split_at(mid).1.join("/"),
                file_name
            )?;
            Ok(new_path)
        }
    }

    /// Returns the relative path to the root of the project.
    ///
    /// Example:
    /// ```
    /// let current_location = "project_root/module/submodule1/submodule2/struct.Name.html";
    /// let result           = "../..";
    /// ```
    /// In this case the first module to match is "module", so we have no need to go back further than that.
    pub(crate) fn path_to_root(&self) -> String {
        (0..self.module_prefixes.len())
            .map(|_| "..")
            .collect::<Vec<_>>()
            .join("/")
    }

    /// Create a path `&str` for navigation from the `module.depth()` & `file_name`.
    ///
    /// This is only used for shorthand path syntax, e.g `../../file_name.html`.
    pub(crate) fn to_html_shorthand_path_string(&self, file_name: &str) -> String {
        format!("{}{}", self.to_html_path_prefix(), file_name)
    }
    /// Create a path prefix `&str` for navigation from the `module.depth()`.
    fn to_html_path_prefix(&self) -> String {
        (0..self.depth()).map(|_| "../").collect::<String>()
    }
    /// The depth of a module as `usize`.
    pub(crate) fn depth(&self) -> usize {
        self.module_prefixes.len()
    }
    /// Create a new [ModuleInfo] from a `TyModule`.
    pub(crate) fn from_ty_module(module_prefixes: Vec<String>, attributes: Option<String>) -> Self {
        Self {
            module_prefixes,
            attributes,
        }
    }
    /// Create a new [ModuleInfo] from a `CallPath`.
    pub(crate) fn from_call_path(call_path: &CallPath) -> Self {
        let module_prefixes = call_path
            .prefixes
            .iter()
            .map(|p| p.as_str().to_string())
            .collect::<Vec<String>>();
        Self {
            module_prefixes,
            attributes: None,
        }
    }
    /// Create a new [ModuleInfo] from a `&[String]`.
    pub(crate) fn from_vec_str(module_prefixes: &[String]) -> Self {
        Self {
            module_prefixes: module_prefixes.to_owned(),
            attributes: None,
        }
    }
    pub(crate) fn preview_opt(&self) -> Option<String> {
        create_preview(self.attributes.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::ModuleInfo;

    #[test]
    fn test_parent() {
        let project = String::from("project_name");
        let module = String::from("module_name");
        let mut module_vec = vec![project.clone(), module];

        let module_info = ModuleInfo::from_ty_module(module_vec.clone(), None);
        let project_opt = module_info.parent();
        assert_eq!(Some(&project), project_opt);

        module_vec.pop();
        let module_info = ModuleInfo::from_ty_module(module_vec, None);
        let project_opt = module_info.parent();
        assert_eq!(None, project_opt);
    }
}
