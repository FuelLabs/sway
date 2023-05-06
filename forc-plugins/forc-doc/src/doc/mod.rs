use crate::{
    doc::{descriptor::Descriptor, module::ModuleInfo},
    render::{item::components::*, link::DocLink, util::format::docstring::*},
};
use anyhow::Result;
use std::option::Option;
use sway_core::{
    decl_engine::DeclEngine,
    language::ty::{TyAstNodeContent, TyDecl, TyImplTrait, TyModule, TyProgram, TySubmodule},
};
use sway_types::Spanned;

mod descriptor;
pub mod module;

#[derive(Default)]
pub(crate) struct Documentation(pub(crate) Vec<Document>);
impl Documentation {
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
        let mut impl_traits: Vec<TyImplTrait> = Vec::new();
        let module_info = ModuleInfo::from_ty_module(vec![project_name.to_owned()], None);
        Documentation::from_ty_module(
            decl_engine,
            module_info,
            &typed_program.root,
            &mut docs,
            &mut impl_traits,
            document_private_items,
        )?;

        // this is the same process as before but for submodules
        for (_, ref typed_submodule) in &typed_program.root.submodules {
            let attributes = (!typed_submodule.module.attributes.is_empty())
                .then(|| typed_submodule.module.attributes.to_html_string());
            let module_prefix =
                ModuleInfo::from_ty_module(vec![project_name.to_owned()], attributes);
            Documentation::from_ty_submodule(
                decl_engine,
                typed_submodule,
                &mut docs,
                &mut impl_traits,
                &module_prefix,
                document_private_items,
                no_deps,
            )?;
        }

        // match for the spans to add the impl_traits to their corresponding doc:
        // currently this compares the spans as str, but this needs to change
        // to compare the actual types
        if !impl_traits.is_empty() {
            for doc in &mut docs.0 {
                let mut impl_vec: Vec<TyImplTrait> = Vec::new();

                match doc.item_body.ty_decl {
                    TyDecl::StructDecl(ref struct_decl) => {
                        for impl_trait in &impl_traits {
                            if struct_decl.name.as_str()
                                == impl_trait.implementing_for.span.as_str()
                                && struct_decl.name.as_str()
                                    != impl_trait.trait_name.suffix.span().as_str()
                            {
                                impl_vec.push(impl_trait.clone());
                            }
                        }
                    }
                    _ => continue,
                }

                if !impl_vec.is_empty() {
                    doc.item_body.item_context.impl_traits = Some(impl_vec);
                }
            }
        }

        Ok(docs)
    }
    fn from_ty_module(
        decl_engine: &DeclEngine,
        module_info: ModuleInfo,
        ty_module: &TyModule,
        docs: &mut Documentation,
        impl_traits: &mut Vec<TyImplTrait>,
        document_private_items: bool,
    ) -> Result<()> {
        for ast_node in &ty_module.all_nodes {
            if let TyAstNodeContent::Declaration(ref decl) = ast_node.content {
                if let TyDecl::ImplTrait(impl_trait) = decl {
                    impl_traits.push(decl_engine.get_impl_trait(&impl_trait.decl_id))
                } else {
                    let desc = Descriptor::from_typed_decl(
                        decl_engine,
                        decl,
                        module_info.clone(),
                        document_private_items,
                    )?;

                    if let Descriptor::Documentable(doc) = desc {
                        docs.0.push(doc)
                    }
                }
            }
        }

        Ok(())
    }
    fn from_ty_submodule(
        decl_engine: &DeclEngine,
        typed_submodule: &TySubmodule,
        docs: &mut Documentation,
        impl_traits: &mut Vec<TyImplTrait>,
        module_info: &ModuleInfo,
        document_private_items: bool,
        no_deps: bool,
    ) -> Result<()> {
        let mut module_info = module_info.to_owned();
        module_info
            .module_prefixes
            .push(typed_submodule.mod_name_span.as_str().to_owned());
        Documentation::from_ty_module(
            decl_engine,
            module_info.clone(),
            &typed_submodule.module,
            docs,
            impl_traits,
            document_private_items,
        )?;
        // if !no_deps {
        //     println!("dep: {:#?}", typed_submodule.module.namespace);
        // }

        for (_, submodule) in &typed_submodule.module.submodules {
            Documentation::from_ty_submodule(
                decl_engine,
                submodule,
                docs,
                impl_traits,
                &module_info,
                document_private_items,
                no_deps,
            )?;
        }

        Ok(())
    }
}
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
}
