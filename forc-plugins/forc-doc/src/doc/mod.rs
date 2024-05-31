//! Handles conversion of compiled typed Sway programs into [Document]s that can be rendered into HTML.
use crate::{
    doc::{descriptor::Descriptor, module::ModuleInfo},
    render::{
        item::{components::*, context::DocImplTrait, documentable_type::DocumentableType},
        link::DocLink,
        util::format::docstring::{create_preview, DocStrings},
    },
};
use anyhow::Result;
use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    option::Option,
};
use sway_core::{
    decl_engine::DeclEngine,
    language::ty::{TyAstNodeContent, TyDecl, TyImplTrait, TyModule, TyProgram, TySubmodule},
    Engines,
};
use sway_types::BaseIdent;

mod descriptor;
pub mod module;

#[derive(Default, Clone)]
pub struct Documentation(pub Vec<Document>);

impl Documentation {
    /// Gather [Documentation] from the [TyProgram].
    pub fn from_ty_program(
        engines: &Engines,
        project_name: &str,
        typed_program: &TyProgram,
        document_private_items: bool,
    ) -> Result<Documentation> {
        // the first module prefix will always be the project name
        let mut docs = Documentation::default();
        let mut impl_traits: Vec<(TyImplTrait, ModuleInfo)> = Vec::new();
        let module_info = ModuleInfo::from_ty_module(vec![project_name.to_owned()], None);
        Documentation::from_ty_module(
            engines.de(),
            &module_info,
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
                engines.de(),
                typed_submodule,
                &mut docs,
                &mut impl_traits,
                &module_prefix,
                document_private_items,
            )?;
        }
        let trait_decls = docs
            .iter()
            .filter_map(|d| {
                (d.item_header.friendly_name == "trait").then_some((
                    d.item_header.item_name.clone(),
                    d.item_header.module_info.clone(),
                ))
            })
            .collect::<HashMap<BaseIdent, ModuleInfo>>();

        // Add one documentation page for each primitive type that has an implementation.
        for (impl_trait, module_info) in impl_traits.iter() {
            let impl_for_type = engines.te().get(impl_trait.implementing_for.type_id);
            if let Ok(Descriptor::Documentable(doc)) =
                Descriptor::from_type_info(impl_for_type.as_ref(), engines, module_info.clone())
            {
                if !docs.iter().any(|existing_doc| *existing_doc == doc) {
                    docs.push(doc);
                }
            }
        }

        // match for the spans to add the impl_traits to their corresponding doc:
        // currently this compares the spans as str, but this needs to change
        // to compare the actual types
        for doc in docs.iter_mut() {
            let mut impl_trait_vec: Vec<DocImplTrait> = Vec::new();
            let mut inherent_impl_vec: Vec<DocImplTrait> = Vec::new();

            // Check for implementations of the current struct/enum/primitive.
            match doc.item_body.ty {
                DocumentableType::Declared(TyDecl::StructDecl(_))
                | DocumentableType::Declared(TyDecl::EnumDecl(_))
                | DocumentableType::Primitive(_) => {
                    let item_name = doc.item_header.item_name.as_str().to_string();
                    for (impl_trait, _) in impl_traits.iter_mut() {
                        // Check if this implementation is for this struct/enum.
                        if item_name.as_str() == impl_trait.implementing_for.span.as_str() {
                            let module_info_override = if let Some(decl_module_info) =
                                trait_decls.get(&impl_trait.trait_name.suffix)
                            {
                                Some(decl_module_info.module_prefixes.clone())
                            } else {
                                impl_trait.trait_name = impl_trait
                                    .trait_name
                                    .to_fullpath(engines, &typed_program.root.namespace);
                                None
                            };

                            let doc_impl_trait = DocImplTrait {
                                impl_for_module: doc.module_info.clone(),
                                impl_trait: impl_trait.clone(),
                                module_info_override,
                            };

                            if doc_impl_trait.is_inherent() {
                                inherent_impl_vec.push(doc_impl_trait);
                            } else {
                                impl_trait_vec.push(doc_impl_trait);
                            }
                        }
                    }
                }
                _ => {}
            }

            if !impl_trait_vec.is_empty() {
                doc.item_body.item_context.impl_traits = Some(impl_trait_vec);
            }

            if !inherent_impl_vec.is_empty() {
                doc.item_body.item_context.inherent_impls = Some(inherent_impl_vec);
            }
        }

        Ok(docs)
    }
    fn from_ty_module(
        decl_engine: &DeclEngine,
        module_info: &ModuleInfo,
        ty_module: &TyModule,
        docs: &mut Documentation,
        impl_traits: &mut Vec<(TyImplTrait, ModuleInfo)>,
        document_private_items: bool,
    ) -> Result<()> {
        for ast_node in &ty_module.all_nodes {
            if let TyAstNodeContent::Declaration(ref decl) = ast_node.content {
                if let TyDecl::ImplTrait(impl_trait) = decl {
                    impl_traits.push((
                        (*decl_engine.get_impl_trait(&impl_trait.decl_id)).clone(),
                        module_info.clone(),
                    ));
                } else {
                    let desc = Descriptor::from_typed_decl(
                        decl_engine,
                        decl,
                        module_info.clone(),
                        document_private_items,
                    )?;

                    if let Descriptor::Documentable(doc) = desc {
                        docs.push(doc);
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
        impl_traits: &mut Vec<(TyImplTrait, ModuleInfo)>,
        module_info: &ModuleInfo,
        document_private_items: bool,
    ) -> Result<()> {
        let mut module_info = module_info.to_owned();
        module_info
            .module_prefixes
            .push(typed_submodule.mod_name_span.as_str().to_owned());
        Documentation::from_ty_module(
            decl_engine,
            &module_info.clone(),
            &typed_submodule.module,
            docs,
            impl_traits,
            document_private_items,
        )?;

        for (_, submodule) in &typed_submodule.module.submodules {
            Documentation::from_ty_submodule(
                decl_engine,
                submodule,
                docs,
                impl_traits,
                &module_info,
                document_private_items,
            )?;
        }

        Ok(())
    }
}

/// A finalized Document ready to be rendered. We want to retain all
/// information including spans, fields on structs, variants on enums etc.
#[derive(Clone, Debug)]
pub struct Document {
    pub module_info: ModuleInfo,
    pub item_header: ItemHeader,
    pub item_body: ItemBody,
    pub raw_attributes: Option<String>,
}

impl Document {
    /// Creates an HTML file name from the [Document].
    pub fn html_filename(&self) -> String {
        use sway_core::language::ty::TyDecl::StorageDecl;
        let name = match &self.item_body.ty {
            &DocumentableType::Declared(StorageDecl { .. }) => None,
            _ => Some(self.item_header.item_name.as_str()),
        };

        Document::create_html_filename(self.item_body.ty.doc_name(), name)
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
    pub fn link(&self) -> DocLink {
        DocLink {
            name: self.item_header.item_name.as_str().to_owned(),
            module_info: self.module_info.clone(),
            html_filename: self.html_filename(),
            preview_opt: self.preview_opt(),
        }
    }
    pub fn preview_opt(&self) -> Option<String> {
        create_preview(self.raw_attributes.clone())
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Self) -> bool {
        self.item_header.item_name == other.item_header.item_name
            && self.item_header.module_info.module_prefixes
                == other.item_header.module_info.module_prefixes
    }
}

impl Deref for Documentation {
    type Target = Vec<Document>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Documentation {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
