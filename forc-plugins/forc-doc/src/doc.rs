use crate::{
    descriptor::Descriptor,
    render::{ItemBody, ItemHeader},
};
use anyhow::Result;
use horrorshow::{box_html, RenderBox};
use sway_core::language::ty::{TyAstNodeContent, TyProgram, TySubmodule};

pub(crate) type Documentation = Vec<Document>;
/// A finalized Document ready to be rendered. We want to retain all
/// information including spans, fields on structs, variants on enums etc.
#[derive(Clone)]
pub(crate) struct Document {
    pub(crate) module_prefix: Vec<String>,
    pub(crate) item_header: ItemHeader,
    pub(crate) item_body: ItemBody,
}
impl Document {
    /// Creates an HTML file name from the [Document].
    pub(crate) fn file_name(&self) -> String {
        use sway_core::language::ty::TyDeclaration::StorageDeclaration;
        let name = match &self.item_body.ty_decl {
            StorageDeclaration(_) => None,
            _ => Some(&self.item_header.item_name),
        };

        Document::create_html_file_name(self.item_body.ty_decl.doc_name(), name.map(|s| &**s))
    }
    fn create_html_file_name(ty: &str, name: Option<&str>) -> String {
        match name {
            Some(name) => {
                format!("{ty}.{name}.html")
            }
            None => {
                format!("{ty}.html") // storage does not have an Ident
            }
        }
    }
    /// Gather [Documentation] from the [TyProgram].
    pub(crate) fn from_ty_program(
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
                    decl,
                    vec![project_name.to_owned()],
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
                let module_prefix = vec![project_name.to_owned()];
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
        module_prefix: &[String],
        document_private_items: bool,
    ) -> Result<()> {
        let mut new_submodule_prefix = module_prefix.to_owned();
        new_submodule_prefix.push(typed_submodule.library_name.as_str().to_string());
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
impl crate::render::Renderable for Document {
    fn render(self) -> Box<dyn RenderBox> {
        box_html! {
            : self.item_header.render();
            : self.item_body.render();
        }
    }
}
