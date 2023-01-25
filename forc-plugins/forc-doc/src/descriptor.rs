//! Determine whether a [Declaration] is documentable.
use crate::{
    doc::{Document, ModuleInfo},
    render::{trim_fn_body, ContextType, DocStrings, ItemBody, ItemContext, ItemHeader},
};
use anyhow::Result;
use sway_core::{
    decl_engine::*,
    language::ty::{TyDeclaration, TyTraitFn},
};
use sway_types::Spanned;

trait RequiredMethods {
    fn to_methods(&self, decl_engine: &DeclEngine) -> Vec<TyTraitFn>;
}
impl RequiredMethods for Vec<sway_core::decl_engine::DeclId> {
    fn to_methods(&self, decl_engine: &DeclEngine) -> Vec<TyTraitFn> {
        self.iter()
            .map(|decl_id| {
                decl_engine
                    .get_trait_fn(decl_id.clone(), &decl_id.span())
                    .expect("could not get trait fn from declaration id")
            })
            .collect()
    }
}

/// Used in deciding whether or not a [Declaration] is documentable.
pub(crate) enum Descriptor {
    Documentable(Document),
    NonDocumentable,
}

impl Descriptor {
    /// Decides whether a [TyDeclaration] is [Descriptor::Documentable].
    pub(crate) fn from_typed_decl(
        decl_engine: &DeclEngine,
        ty_decl: &TyDeclaration,
        module_info: ModuleInfo,
        document_private_items: bool,
    ) -> Result<Self> {
        const CONTRACT_STORAGE: &str = "Contract Storage";

        use swayfmt::parse;
        use TyDeclaration::*;
        match ty_decl {
            StructDeclaration(ref decl_id) => {
                let struct_decl = decl_engine.get_struct(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && struct_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = struct_decl.name;
                    let (attrs_opt, raw_attributes) = match struct_decl.attributes.is_empty() {
                        true => (None, None),
                        false => (
                            Some(struct_decl.attributes.to_html_string()),
                            Some(struct_decl.attributes.to_raw_string()),
                        ),
                    };
                    let context = (!struct_decl.fields.is_empty())
                        .then_some(ContextType::StructFields(struct_decl.fields));

                    Ok(Descriptor::Documentable(Document {
                        module_info: module_info.clone(),
                        item_header: ItemHeader {
                            module_info: module_info.clone(),
                            friendly_name: ty_decl.friendly_name(),
                            item_name: item_name.clone(),
                        },
                        item_body: ItemBody {
                            module_info,
                            ty_decl: ty_decl.clone(),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemStruct>(
                                struct_decl.span.as_str(),
                            ),
                            attrs_opt,
                            item_context: ItemContext { context },
                        },
                        raw_attributes,
                    }))
                }
            }
            EnumDeclaration(ref decl_id) => {
                let enum_decl = decl_engine.get_enum(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && enum_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = enum_decl.name;
                    let (attrs_opt, raw_attributes) = match enum_decl.attributes.is_empty() {
                        true => (None, None),
                        false => (
                            Some(enum_decl.attributes.to_html_string()),
                            Some(enum_decl.attributes.to_raw_string()),
                        ),
                    };
                    let context = (!enum_decl.variants.is_empty())
                        .then_some(ContextType::EnumVariants(enum_decl.variants));

                    Ok(Descriptor::Documentable(Document {
                        module_info: module_info.clone(),
                        item_header: ItemHeader {
                            module_info: module_info.clone(),
                            friendly_name: ty_decl.friendly_name(),
                            item_name: item_name.clone(),
                        },
                        item_body: ItemBody {
                            module_info,
                            ty_decl: ty_decl.clone(),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemEnum>(
                                enum_decl.span.as_str(),
                            ),
                            attrs_opt,
                            item_context: ItemContext { context },
                        },
                        raw_attributes,
                    }))
                }
            }
            TraitDeclaration(ref decl_id) => {
                let trait_decl = decl_engine.get_trait(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && trait_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = trait_decl.name;
                    let (attrs_opt, raw_attributes) = match trait_decl.attributes.is_empty() {
                        true => (None, None),
                        false => (
                            Some(trait_decl.attributes.to_html_string()),
                            Some(trait_decl.attributes.to_raw_string()),
                        ),
                    };
                    let context = (!trait_decl.interface_surface.is_empty()).then_some(
                        ContextType::RequiredMethods(
                            trait_decl.interface_surface.to_methods(decl_engine),
                        ),
                    );

                    Ok(Descriptor::Documentable(Document {
                        module_info: module_info.clone(),
                        item_header: ItemHeader {
                            module_info: module_info.clone(),
                            friendly_name: ty_decl.friendly_name(),
                            item_name: item_name.clone(),
                        },
                        item_body: ItemBody {
                            module_info,
                            ty_decl: ty_decl.clone(),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemTrait>(
                                trait_decl.span.as_str(),
                            ),
                            attrs_opt,
                            item_context: ItemContext { context },
                        },
                        raw_attributes,
                    }))
                }
            }
            AbiDeclaration(ref decl_id) => {
                let abi_decl = decl_engine.get_abi(decl_id.clone(), &decl_id.span())?;
                let item_name = abi_decl.name;
                let (attrs_opt, raw_attributes) = match abi_decl.attributes.is_empty() {
                    true => (None, None),
                    false => (
                        Some(abi_decl.attributes.to_html_string()),
                        Some(abi_decl.attributes.to_raw_string()),
                    ),
                };
                let context = (!abi_decl.interface_surface.is_empty()).then_some(
                    ContextType::RequiredMethods(
                        abi_decl.interface_surface.to_methods(decl_engine),
                    ),
                );

                Ok(Descriptor::Documentable(Document {
                    module_info: module_info.clone(),
                    item_header: ItemHeader {
                        module_info: module_info.clone(),
                        friendly_name: ty_decl.friendly_name(),
                        item_name: item_name.clone(),
                    },
                    item_body: ItemBody {
                        module_info,
                        ty_decl: ty_decl.clone(),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemAbi>(abi_decl.span.as_str()),
                        attrs_opt,
                        item_context: ItemContext { context },
                    },
                    raw_attributes,
                }))
            }
            StorageDeclaration(ref decl_id) => {
                let storage_decl = decl_engine.get_storage(decl_id.clone(), &decl_id.span())?;
                let item_name = sway_types::BaseIdent::new_no_trim(
                    sway_types::span::Span::from_string(CONTRACT_STORAGE.to_string()),
                );
                let (attrs_opt, raw_attributes) = match storage_decl.attributes.is_empty() {
                    true => (None, None),
                    false => (
                        Some(storage_decl.attributes.to_html_string()),
                        Some(storage_decl.attributes.to_raw_string()),
                    ),
                };
                let context = (!storage_decl.fields.is_empty())
                    .then_some(ContextType::StorageFields(storage_decl.fields));

                Ok(Descriptor::Documentable(Document {
                    module_info: module_info.clone(),
                    item_header: ItemHeader {
                        module_info: module_info.clone(),
                        friendly_name: ty_decl.friendly_name(),
                        item_name: item_name.clone(),
                    },
                    item_body: ItemBody {
                        module_info,
                        ty_decl: ty_decl.clone(),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemStorage>(
                            storage_decl.span.as_str(),
                        ),
                        attrs_opt,
                        item_context: ItemContext { context },
                    },
                    raw_attributes,
                }))
            }
            ImplTrait(ref decl_id) => {
                // TODO: figure out how to use this, likely we don't want to document this directly.
                //
                // This declaration type may make more sense to document as part of another declaration
                // much like how we document method functions for traits or fields on structs.
                let impl_trait = decl_engine.get_impl_trait(decl_id.clone(), &decl_id.span())?;
                let item_name = impl_trait.trait_name.suffix;
                Ok(Descriptor::Documentable(Document {
                    module_info: module_info.clone(),
                    item_header: ItemHeader {
                        module_info: module_info.clone(),
                        friendly_name: ty_decl.friendly_name(),
                        item_name: item_name.clone(),
                    },
                    item_body: ItemBody {
                        module_info,
                        ty_decl: ty_decl.clone(),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemImpl>(
                            impl_trait.span.as_str(),
                        ),
                        attrs_opt: None, // no attributes field
                        item_context: ItemContext { context: None },
                    },
                    raw_attributes: None,
                }))
            }
            FunctionDeclaration(ref decl_id) => {
                let fn_decl = decl_engine.get_function(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && fn_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = fn_decl.name;
                    let (attrs_opt, raw_attributes) = match fn_decl.attributes.is_empty() {
                        true => (None, None),
                        false => (
                            Some(fn_decl.attributes.to_html_string()),
                            Some(fn_decl.attributes.to_raw_string()),
                        ),
                    };

                    Ok(Descriptor::Documentable(Document {
                        module_info: module_info.clone(),
                        item_header: ItemHeader {
                            module_info: module_info.clone(),
                            friendly_name: ty_decl.friendly_name(),
                            item_name: item_name.clone(),
                        },
                        item_body: ItemBody {
                            module_info,
                            ty_decl: ty_decl.clone(),
                            item_name,
                            code_str: trim_fn_body(parse::parse_format::<sway_ast::ItemFn>(
                                fn_decl.span.as_str(),
                            )),
                            attrs_opt,
                            item_context: ItemContext { context: None },
                        },
                        raw_attributes,
                    }))
                }
            }
            ConstantDeclaration(ref decl_id) => {
                let const_decl = decl_engine.get_constant(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && const_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = const_decl.name;
                    let (attrs_opt, raw_attributes) = match const_decl.attributes.is_empty() {
                        true => (None, None),
                        false => (
                            Some(const_decl.attributes.to_html_string()),
                            Some(const_decl.attributes.to_raw_string()),
                        ),
                    };

                    Ok(Descriptor::Documentable(Document {
                        module_info: module_info.clone(),
                        item_header: ItemHeader {
                            module_info: module_info.clone(),
                            friendly_name: ty_decl.friendly_name(),
                            item_name: item_name.clone(),
                        },
                        item_body: ItemBody {
                            module_info,
                            ty_decl: ty_decl.clone(),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemConst>(
                                const_decl.span.as_str(),
                            ),
                            attrs_opt,
                            item_context: ItemContext { context: None },
                        },
                        raw_attributes,
                    }))
                }
            }
            _ => Ok(Descriptor::NonDocumentable),
        }
    }
}
