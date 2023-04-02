use crate::{
    descriptor::Descriptor,
    render::{
        split_at_markdown_header, DocLink, DocStrings, ItemBody, ItemHeader, Renderable,
        INDEX_FILENAME,
    },
    RenderPlan,
};
use anyhow::Result;
use horrorshow::{box_html, RenderBox, Template};
use std::{fmt::Write, option::Option, path::PathBuf};
use sway_core::{
    decl_engine::DeclEngine,
    language::{
        ty::{TyAstNodeContent, TyProgram, TySubmodule},
        CallPath,
    },
};

pub(crate) type Documentation = Vec<Document>;
/// A finalized Document ready to be rendered. We want to retain all
/// information including spans, fields on structs, variants on enums etc.
#[derive(Clone, Debug)]
pub(crate) struct Document {
    pub(crate) module_info: ModuleInfo,
    pub(crate) item_header: ItemHeader,
    pub(crate) item_body: ItemBody,
    pub(crate) raw_attributes: Option<String>,
}
impl Document {
    /// Creates an HTML file name from the [Document].
    pub(crate) fn html_filename(&self) -> String {
        use sway_core::language::ty::TyDecl::StorageDecl;
        let name = match &self.item_body.ty_decl {
            StorageDecl { .. } => None,
            _ => Some(self.item_header.item_name.as_str()),
        };

        Document::create_html_filename(self.item_body.ty_decl.doc_name(), name)
    }
    fn create_html_filename(ty: &str, name: Option<&str>) -> String {
        match name {
            Some(name) => format!("{ty}.{name}.html"),
            None => {
                format!("{ty}.html") // storage does not have an Ident
            }
        }
    }
    /// Generate link info used in navigation between docs.
    pub(crate) fn link(&self) -> DocLink {
        DocLink {
            name: self.item_header.item_name.as_str().to_owned(),
            module_info: self.module_info.clone(),
            html_filename: self.html_filename(),
            preview_opt: self.preview_opt(),
        }
    }
    fn preview_opt(&self) -> Option<String> {
        create_preview(self.raw_attributes.clone())
    }
    /// Gather [Documentation] from the [TyProgram].
    pub(crate) fn from_ty_program(
        decl_engine: &DeclEngine,
        project_name: &str,
        typed_program: &TyProgram,
        no_deps: bool,
        document_private_items: bool,
    ) -> Result<Documentation> {
        // the first module prefix will always be the project name
        let mut docs: Documentation = Default::default();
        for ast_node in &typed_program.root.all_nodes {
            if let TyAstNodeContent::Declaration(ref decl) = ast_node.content {
                let desc = Descriptor::from_typed_decl(
                    decl_engine,
                    decl,
                    ModuleInfo::from_ty_module(vec![project_name.to_owned()], None),
                    document_private_items,
                )?;

                if let Descriptor::Documentable(doc) = desc {
                    docs.push(doc)
                }
            }
        }

        if !no_deps && !typed_program.root.submodules.is_empty() {
            // this is the same process as before but for dependencies
            for (_, ref typed_submodule) in &typed_program.root.submodules {
                let attributes = (!typed_submodule.module.attributes.is_empty())
                    .then(|| typed_submodule.module.attributes.to_html_string());
                let module_prefix =
                    ModuleInfo::from_ty_module(vec![project_name.to_owned()], attributes);
                Document::from_ty_submodule(
                    decl_engine,
                    typed_submodule,
                    &mut docs,
                    &module_prefix,
                    document_private_items,
                )?;
            }
        }

        Ok(docs)
    }
    fn from_ty_submodule(
        decl_engine: &DeclEngine,
        typed_submodule: &TySubmodule,
        docs: &mut Documentation,
        module_prefix: &ModuleInfo,
        document_private_items: bool,
    ) -> Result<()> {
        let mut new_submodule_prefix = module_prefix.to_owned();
        new_submodule_prefix
            .module_prefixes
            .push(typed_submodule.mod_name_span.as_str().to_owned());
        for ast_node in &typed_submodule.module.all_nodes {
            if let TyAstNodeContent::Declaration(ref decl) = ast_node.content {
                let desc = Descriptor::from_typed_decl(
                    decl_engine,
                    decl,
                    new_submodule_prefix.clone(),
                    document_private_items,
                )?;

                if let Descriptor::Documentable(doc) = desc {
                    docs.push(doc)
                }
            }
        }
        for (_, submodule) in &typed_submodule.module.submodules {
            Document::from_ty_submodule(
                decl_engine,
                submodule,
                docs,
                &new_submodule_prefix,
                document_private_items,
            )?;
        }

        Ok(())
    }
}
impl Renderable for Document {
    fn render(self, render_plan: RenderPlan) -> Result<Box<dyn RenderBox>> {
        let header = self.item_header.render(render_plan.clone())?;
        let body = self.item_body.render(render_plan)?;
        Ok(box_html! {
            : header;
            : body;
        })
    }
}

pub(crate) type ModulePrefixes = Vec<String>;
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct ModuleInfo {
    pub(crate) module_prefixes: ModulePrefixes,
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
        match self.has_parent() {
            true => {
                let mut iter = self.module_prefixes.iter();
                iter.next_back();
                iter.next_back()
            }
            false => None,
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
        iter.map(|s| s.as_str()).collect::<Vec<&str>>().join("::")
    }
    /// Renders the [ModuleInfo] into a [CallPath] with anchors. We return this as a `Result<Vec<String>>`
    /// since the `box_html!` macro returns a closure and no two closures are considered the same type.
    pub(crate) fn get_anchors(&self) -> Result<Vec<String>> {
        let mut count = self.depth();
        let mut rendered_module_anchors = Vec::with_capacity(self.depth());
        for prefix in self.module_prefixes.iter() {
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
    /// current_location = "project_root/module/submodule1/submodule2/struct.Name.html";
    /// next_location    =              "module/other_submodule/enum.Name.html";
    /// result           =               "../../other_submodule/enum.Name.html";
    /// ```
    /// In this case the first module to match is "module", so we have no need to go back further than that.
    pub(crate) fn file_path_from_location(
        &self,
        file_name: &str,
        current_module_info: &ModuleInfo,
    ) -> Result<String> {
        let mut mid = 0; // the index to split the module_info from call_path at
        let mut offset = 0; // the number of directories to go back
        let mut next_location_iter = self.module_prefixes.iter().rev().enumerate().peekable();
        while let Some((index, prefix)) = next_location_iter.peek() {
            for (count, module) in current_module_info.module_prefixes.iter().rev().enumerate() {
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
    /// Create a path `&str` for navigation from the `module.depth()` & `file_name`.
    ///
    /// This is only used for shorthand path syntax, e.g `../../file_name.html`.
    pub(crate) fn to_html_shorthand_path_string(&self, file_name: &str) -> String {
        format!("{}{}", self.to_html_path_prefix(), file_name)
    }
    /// Create a path prefix `&str` for navigation from the `module.depth()`.
    fn to_html_path_prefix(&self) -> String {
        (1..self.depth()).map(|_| "../").collect::<String>()
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
    pub(crate) fn from_call_path(call_path: CallPath) -> Self {
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
    pub(crate) fn preview_opt(&self) -> Option<String> {
        create_preview(self.attributes.clone())
    }
}

/// Create a docstring preview from raw html attributes.
///
/// Returns `None` if there are no attributes.
fn create_preview(raw_attributes: Option<String>) -> Option<String> {
    const MAX_PREVIEW_CHARS: usize = 100;
    const CLOSING_PARAGRAPH_TAG: &str = "</p>";

    raw_attributes.as_ref().map(|description| {
        let preview = split_at_markdown_header(description);
        if preview.chars().count() > MAX_PREVIEW_CHARS && preview.contains(CLOSING_PARAGRAPH_TAG) {
            let closing_tag_index = preview
                .find(CLOSING_PARAGRAPH_TAG)
                .expect("closing tag out of range");
            // We add 1 here to get the index of the char after the closing tag.
            // This ensures we retain the closing tag and don't break the html.
            let (preview, _) =
                preview.split_at(closing_tag_index + CLOSING_PARAGRAPH_TAG.len() + 1);
            if preview.chars().count() > MAX_PREVIEW_CHARS && preview.contains('\n') {
                let newline_index = preview.find('\n').expect("new line char out of range");
                preview.split_at(newline_index).0.to_string()
            } else {
                preview.to_string()
            }
        } else {
            preview.to_string()
        }
    })
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
