use std::path::PathBuf;

use crate::{
    descriptor::Descriptor,
    render::{ItemBody, ItemHeader, Renderable},
};
use anyhow::Result;
use horrorshow::{box_html, RenderBox};
use sway_core::language::ty::{TyAstNodeContent, TyProgram, TySubmodule};

pub(crate) type Documentation<'dir, 'mdl_info> = Vec<Document<'dir, 'mdl_info>>;
/// A finalized Document ready to be rendered. We want to retain all
/// information including spans, fields on structs, variants on enums etc.
#[derive(Clone)]
pub(crate) struct Document<'dir, 'mdl_info> {
    pub(crate) module_info: &'dir ModuleInfo<'mdl_info>,
    pub(crate) item_header: ItemHeader<'mdl_info>,
    pub(crate) item_body: ItemBody<'mdl_info>,
}
impl<'dir> Document<'_, 'dir> {
    /// Creates an HTML file name from the [Document].
    pub(crate) fn html_file_name(&self) -> &str {
        use sway_core::language::ty::TyDeclaration::StorageDeclaration;
        let name = match &self.item_body.ty_decl {
            StorageDeclaration(_) => None,
            _ => Some(&self.item_header.item_name),
        };

        Document::create_html_file_name(self.item_body.ty_decl.doc_name(), name.map(|s| &**s))
    }
    fn create_html_file_name<'name>(ty: &'name str, name: Option<&'name str>) -> &'name str {
        match name {
            Some(name) => &format!("{ty}.{name}.html"),
            None => {
                &format!("{ty}.html") // storage does not have an Ident
            }
        }
    }
    /// Gather [Documentation] from the [TyProgram].
    pub(crate) fn from_ty_program<'proj_name>(
        project_name: &'proj_name str,
        typed_program: &TyProgram,
        no_deps: bool,
        document_private_items: bool,
    ) -> Result<Documentation<'dir, 'proj_name>> {
        // the first module prefix will always be the project name
        let mut docs: Documentation = Default::default();
        for ast_node in &typed_program.root.all_nodes {
            if let TyAstNodeContent::Declaration(ref decl) = ast_node.content {
                let desc = Descriptor::from_typed_decl(
                    decl,
                    ModuleInfo::from_vec(vec![project_name]),
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
                let module_prefix = ModuleInfo::from_vec(vec![project_name]);
                Document::from_ty_submodule(
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
        typed_submodule: &TySubmodule,
        docs: &mut Documentation,
        module_prefix: &ModuleInfo,
        document_private_items: bool,
    ) -> Result<()> {
        let mut new_submodule_prefix = module_prefix.to_owned();
        new_submodule_prefix
            .0
            .push(typed_submodule.library_name.as_str());
        for ast_node in &typed_submodule.module.all_nodes {
            if let TyAstNodeContent::Declaration(ref decl) = ast_node.content {
                let desc = Descriptor::from_typed_decl(
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
                submodule,
                docs,
                &new_submodule_prefix,
                document_private_items,
            )?;
        }

        Ok(())
    }
}
impl<'dir> Renderable for Document<'_, 'dir> {
    fn render(self) -> Box<dyn RenderBox> {
        box_html! {
            : self.item_header.render();
            : self.item_body.render();
        }
    }
}
pub(crate) type ModulePrefix<'mdl_info> = &'mdl_info str;
#[derive(Clone)]
pub(crate) struct ModuleInfo<'mdl_info>(pub(crate) Vec<ModulePrefix<'mdl_info>>);
impl ModuleInfo<'_> {
    /// The current prefix.
    pub(crate) fn location(&self) -> &str {
        self.0
            .last()
            .expect("There will always be at least the project name")
    }
    /// The name of the project.
    pub(crate) fn project_name(&self) -> &str {
        self.0.first().expect("Project name missing")
    }
    /// Create a qualified path that represents the full path to an item.
    pub(crate) fn to_path_literal_str(&self) -> &str {
        &self.0.join("::")
    }
    /// Creates a String version of the path to an item,
    /// used in navigation between pages.
    pub(crate) fn to_file_path_str(&self, file_name: &str) -> &str {
        let mut iter = self.0.iter();
        iter.next(); // skip the project_name
        iter.map(|s| *s)
            .collect::<PathBuf>()
            .to_str()
            .expect("There will always be at least the item name")
    }
    /// Create a path `&str` for navigation from the `module.depth()` & `file_name`.
    pub(crate) fn to_html_shorthand_path_str(&self, file_name: &str) -> &str {
        &format!("{}{}", self.to_html_path_prefix(), file_name)
    }
    /// Create a path prefix `&str` for navigation from the `module.depth()`.
    fn to_html_path_prefix(&self) -> &str {
        &(1..self.depth()).map(|_| "../").collect::<String>()
    }
    /// The depth of a module as `usize`.
    pub(crate) fn depth(&self) -> usize {
        self.0.len()
    }
    /// Create a new [ModuleInfo] from a vec.
    fn from_vec(vec: Vec<&str>) -> Self {
        Self(vec)
    }
}
