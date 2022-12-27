use crate::{
    descriptor::Descriptor,
    render::{ItemBody, ItemHeader, Renderable},
};
use anyhow::Result;
use horrorshow::{box_html, RenderBox};
use std::path::PathBuf;
use sway_core::{
    declaration_engine::DeclarationEngine,
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
}
impl Document {
    /// Creates an HTML file name from the [Document].
    pub(crate) fn html_file_name(&self) -> String {
        use sway_core::language::ty::TyDeclaration::StorageDeclaration;
        let name = match &self.item_body.ty_decl {
            StorageDeclaration(_) => None,
            _ => Some(self.item_header.item_name.as_str()),
        };

        Document::create_html_file_name(self.item_body.ty_decl.doc_name(), name)
    }
    fn create_html_file_name<'name>(ty: &'name str, name: Option<&'name str>) -> String {
        match name {
            Some(name) => format!("{ty}.{name}.html"),
            None => {
                format!("{ty}.html") // storage does not have an Ident
            }
        }
    }
    /// Gather [Documentation] from the [TyProgram].
    pub(crate) fn from_ty_program(
        declaration_engine: &DeclarationEngine,
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
                    declaration_engine,
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
                    declaration_engine,
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
        declaration_engine: &DeclarationEngine,
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
                    declaration_engine,
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
                declaration_engine,
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
#[derive(Clone)]
pub(crate) struct ModuleInfo(pub(crate) Vec<ModulePrefix>);
impl ModuleInfo {
    /// The current prefix.
    pub(crate) fn location(&self) -> &str {
        self.0
            .last()
            .expect("There will always be at least the project name")
    }
    /// The name of the project.
    pub(crate) fn _project_name(&self) -> &str {
        self.0.first().expect("Project name missing")
    }
    /// Create a qualified path that represents the full path to an item.
    pub(crate) fn to_path_literal_str(&self) -> String {
        self.0.join("::")
    }
    /// Creates a String version of the path to an item,
    /// used in navigation between pages.
    pub(crate) fn to_file_path_str(&self, file_name: &str) -> String {
        let mut iter = self.0.iter();
        iter.next(); // skip the project_name
        let mut file_path = iter.collect::<PathBuf>();
        file_path.push(file_name);

        file_path
            .to_str()
            .expect("There will always be at least the item name")
            .to_string()
    }
    /// Create a path `&str` for navigation from the `module.depth()` & `file_name`.
    pub(crate) fn to_html_shorthand_path_str(&self, file_name: &str) -> String {
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
    fn from_vec(vec: Vec<String>) -> Self {
        Self(vec)
    }
}
