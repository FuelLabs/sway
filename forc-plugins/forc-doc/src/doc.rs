use crate::{
    descriptor::Descriptor,
    render::{split_at_markdown_header, DocLink, ItemBody, ItemHeader, Renderable},
};
use anyhow::Result;
use horrorshow::{box_html, RenderBox};
use std::path::PathBuf;
use sway_core::{
    decl_engine::DeclEngine,
    language::ty::{TyAstNodeContent, TyProgram, TySubmodule},
};

pub(crate) type Documentation = Vec<Document>;
/// A finalized Document ready to be rendered. We want to retain all
/// information including spans, fields on structs, variants on enums etc.
#[derive(Clone)]
pub(crate) struct Document {
    pub(crate) module_info: ModuleInfo,
    pub(crate) item_header: ItemHeader,
    pub(crate) item_body: ItemBody,
    pub(crate) raw_attributes: Option<String>,
}
impl Document {
    /// Creates an HTML file name from the [Document].
    pub(crate) fn html_filename(&self) -> String {
        use sway_core::language::ty::TyDeclaration::StorageDeclaration;
        let name = match &self.item_body.ty_decl {
            StorageDeclaration(_) => None,
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
        const MAX_PREVIEW_CHARS: usize = 100;
        const CLOSING_PARAGRAPH_TAG: &str = "</p>";

        self.raw_attributes.as_ref().map(|description| {
            let preview = split_at_markdown_header(description);
            if preview.chars().count() > MAX_PREVIEW_CHARS
                && preview.contains(CLOSING_PARAGRAPH_TAG)
            {
                match preview.find(CLOSING_PARAGRAPH_TAG) {
                    Some(index) => {
                        // We add 1 here to get the index of the char after the closing tag.
                        // This ensures we retain the closing tag and don't break the html.
                        let (preview, _) =
                            preview.split_at(index + CLOSING_PARAGRAPH_TAG.len() + 1);
                        if preview.chars().count() > MAX_PREVIEW_CHARS && preview.contains('\n') {
                            match preview.find('\n') {
                                Some(index) => preview.split_at(index).0.to_string(),
                                None => unreachable!("Previous logic prevents this panic"),
                            }
                        } else {
                            preview.to_string()
                        }
                    }
                    None => unreachable!("Previous logic prevents this panic"),
                }
            } else {
                preview.to_string()
            }
        })
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
                    ModuleInfo::from_vec(vec![project_name.to_owned()]),
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
                let module_prefix = ModuleInfo::from_vec(vec![project_name.to_owned()]);
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
            .0
            .push(typed_submodule.library_name.as_str().to_owned());
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
        // if there is another submodule we need to go a level deeper
        if let Some((_, submodule)) = typed_submodule.module.submodules.first() {
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
    fn render(self) -> Box<dyn RenderBox> {
        box_html! {
            : self.item_header.render();
            : self.item_body.render();
        }
    }
}
pub(crate) type ModulePrefix = String;
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) struct ModuleInfo(pub(crate) Vec<ModulePrefix>);
impl ModuleInfo {
    /// The current module.
    pub(crate) fn location(&self) -> &str {
        self.0
            .last()
            .expect("There will always be at least the project name")
    }
    /// The name of the project.
    pub(crate) fn project_name(&self) -> &str {
        self.0.first().expect("Project name missing")
    }
    /// The location of the parent of the current module.
    ///
    /// Returns `None` if there is no parent.
    pub(crate) fn parent(&self) -> Option<&String> {
        match self.has_parent() {
            true => {
                let mut iter = self.0.iter();
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
        let mut iter = self.0.iter();
        for prefix in iter.by_ref() {
            if prefix == location {
                break;
            }
        }
        iter.map(|s| s.as_str()).collect::<Vec<&str>>().join("::")
    }
    /// Creates a String version of the path to an item,
    /// used in navigation between pages.
    ///
    /// This is only used for full path syntax, e.g `module/submodule/file_name.html`.
    pub(crate) fn to_file_path_string(&self, file_name: &str, location: &str) -> String {
        let mut iter = self.0.iter();
        for prefix in iter.by_ref() {
            if prefix == location {
                break;
            }
        }
        let mut file_path = iter.collect::<PathBuf>();
        file_path.push(file_name);

        file_path
            .to_str()
            .expect("There will always be at least the item name")
            .to_string()
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
        self.0.len()
    }
    /// Create a new [ModuleInfo] from a vec.
    pub(crate) fn from_vec(vec: Vec<String>) -> Self {
        Self(vec)
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

        let module_info = ModuleInfo::from_vec(module_vec.clone());
        let project_opt = module_info.parent();
        assert_eq!(Some(&project), project_opt);

        module_vec.pop();
        let module_info = ModuleInfo::from_vec(module_vec);
        let project_opt = module_info.parent();
        assert_eq!(None, project_opt);
    }
}
