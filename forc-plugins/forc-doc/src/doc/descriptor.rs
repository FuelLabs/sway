//! Determine whether a [Declaration] is documentable.
use crate::{
    doc::{module::ModuleInfo, Document},
    render::{
        item::{components::*, context::*},
        util::format::{code_block::trim_fn_body, docstring::DocStrings},
    },
};
use anyhow::Result;
use sway_core::{
    decl_engine::*,
    language::ty::{TyDecl, TyTraitFn, TyTraitInterfaceItem},
};

trait RequiredMethods {
    fn to_methods(&self, decl_engine: &DeclEngine) -> Vec<TyTraitFn>;
}
impl RequiredMethods for Vec<DeclRefTraitFn> {
    fn to_methods(&self, decl_engine: &DeclEngine) -> Vec<TyTraitFn> {
        self.iter()
            .map(|decl_ref| decl_engine.get_trait_fn(decl_ref))
            .collect()
    }
}

/// Used in deciding whether or not a [Declaration] is documentable.
pub(crate) enum Descriptor {
    Documentable(Document),
    NonDocumentable,
}

impl Descriptor {
    /// Decides whether a [TyDecl] is [Descriptor::Documentable] and returns a [Document] if so.
    pub(crate) fn from_typed_decl(
        decl_engine: &DeclEngine,
        ty_decl: &TyDecl,
        module_info: ModuleInfo,
        document_private_items: bool,
    ) -> Result<Self> {
        const CONTRACT_STORAGE: &str = "Contract Storage";

        use swayfmt::parse;
        use TyDecl::*;
        match ty_decl {
            StructDecl { decl_id, .. } => {
                let struct_decl = decl_engine.get_struct(decl_id);
                if !document_private_items && struct_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = struct_decl.call_path.suffix;
                    let attrs_opt = (!struct_decl.attributes.is_empty())
                        .then(|| struct_decl.attributes.to_html_string());
                    let context = (!struct_decl.fields.is_empty()).then_some(Context::new(
                        module_info.clone(),
                        ContextType::StructFields(struct_decl.fields),
                    ));

                    Ok(Descriptor::Documentable(Document {
                        module_info: module_info.clone(),
                        item_header: ItemHeader {
                            module_info: module_info.clone(),
                            friendly_name: ty_decl.friendly_type_name(),
                            item_name: item_name.clone(),
                        },
                        item_body: ItemBody {
                            module_info,
                            ty_decl: ty_decl.clone(),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemStruct>(
                                struct_decl.span.as_str(),
                            ),
                            attrs_opt: attrs_opt.clone(),
                            item_context: ItemContext {
                                context_opt: context,
                                impl_traits: None,
                            },
                        },
                        raw_attributes: attrs_opt,
                    }))
                }
            }
            EnumDecl { decl_id, .. } => {
                let enum_decl = decl_engine.get_enum(decl_id);
                if !document_private_items && enum_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = enum_decl.call_path.suffix;
                    let attrs_opt = (!enum_decl.attributes.is_empty())
                        .then(|| enum_decl.attributes.to_html_string());
                    let context = (!enum_decl.variants.is_empty()).then_some(Context::new(
                        module_info.clone(),
                        ContextType::EnumVariants(enum_decl.variants),
                    ));

                    Ok(Descriptor::Documentable(Document {
                        module_info: module_info.clone(),
                        item_header: ItemHeader {
                            module_info: module_info.clone(),
                            friendly_name: ty_decl.friendly_type_name(),
                            item_name: item_name.clone(),
                        },
                        item_body: ItemBody {
                            module_info,
                            ty_decl: ty_decl.clone(),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemEnum>(
                                enum_decl.span.as_str(),
                            ),
                            attrs_opt: attrs_opt.clone(),
                            item_context: ItemContext {
                                context_opt: context,
                                impl_traits: None,
                            },
                        },
                        raw_attributes: attrs_opt,
                    }))
                }
            }
            TraitDecl { decl_id, .. } => {
                let trait_decl = decl_engine.get_trait(decl_id);
                if !document_private_items && trait_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = trait_decl.name;
                    let attrs_opt = (!trait_decl.attributes.is_empty())
                        .then(|| trait_decl.attributes.to_html_string());
                    let context =
                        (!trait_decl.interface_surface.is_empty()).then_some(Context::new(
                            module_info.clone(),
                            ContextType::RequiredMethods(
                                trait_decl
                                    .interface_surface
                                    .into_iter()
                                    .flat_map(|item| match item {
                                        TyTraitInterfaceItem::TraitFn(fn_decl) => Some(fn_decl),
                                        _ => None,
                                    })
                                    .collect::<Vec<_>>()
                                    .to_methods(decl_engine),
                            ),
                        ));

                    Ok(Descriptor::Documentable(Document {
                        module_info: module_info.clone(),
                        item_header: ItemHeader {
                            module_info: module_info.clone(),
                            friendly_name: ty_decl.friendly_type_name(),
                            item_name: item_name.clone(),
                        },
                        item_body: ItemBody {
                            module_info,
                            ty_decl: ty_decl.clone(),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemTrait>(
                                trait_decl.span.as_str(),
                            ),
                            attrs_opt: attrs_opt.clone(),
                            item_context: ItemContext {
                                context_opt: context,
                                impl_traits: None,
                            },
                        },
                        raw_attributes: attrs_opt,
                    }))
                }
            }
            AbiDecl { decl_id, .. } => {
                let abi_decl = decl_engine.get_abi(decl_id);
                let item_name = abi_decl.name;
                let attrs_opt =
                    (!abi_decl.attributes.is_empty()).then(|| abi_decl.attributes.to_html_string());
                let context = (!abi_decl.interface_surface.is_empty()).then_some(Context::new(
                    module_info.clone(),
                    ContextType::RequiredMethods(
                        abi_decl
                            .interface_surface
                            .into_iter()
                            .flat_map(|item| match item {
                                TyTraitInterfaceItem::TraitFn(fn_decl) => Some(fn_decl),
                                _ => None,
                            })
                            .collect::<Vec<_>>()
                            .to_methods(decl_engine),
                    ),
                ));

                Ok(Descriptor::Documentable(Document {
                    module_info: module_info.clone(),
                    item_header: ItemHeader {
                        module_info: module_info.clone(),
                        friendly_name: ty_decl.friendly_type_name(),
                        item_name: item_name.clone(),
                    },
                    item_body: ItemBody {
                        module_info,
                        ty_decl: ty_decl.clone(),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemAbi>(abi_decl.span.as_str()),
                        attrs_opt: attrs_opt.clone(),
                        item_context: ItemContext {
                            context_opt: context,
                            impl_traits: None,
                        },
                    },
                    raw_attributes: attrs_opt,
                }))
            }
            StorageDecl { decl_id, .. } => {
                let storage_decl = decl_engine.get_storage(decl_id);
                let item_name = sway_types::BaseIdent::new_no_trim(
                    sway_types::span::Span::from_string(CONTRACT_STORAGE.to_string()),
                );
                let attrs_opt = (!storage_decl.attributes.is_empty())
                    .then(|| storage_decl.attributes.to_html_string());
                let context = (!storage_decl.fields.is_empty()).then_some(Context::new(
                    module_info.clone(),
                    ContextType::StorageFields(storage_decl.fields),
                ));

                Ok(Descriptor::Documentable(Document {
                    module_info: module_info.clone(),
                    item_header: ItemHeader {
                        module_info: module_info.clone(),
                        friendly_name: ty_decl.friendly_type_name(),
                        item_name: item_name.clone(),
                    },
                    item_body: ItemBody {
                        module_info,
                        ty_decl: ty_decl.clone(),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemStorage>(
                            storage_decl.span.as_str(),
                        ),
                        attrs_opt: attrs_opt.clone(),
                        item_context: ItemContext {
                            context_opt: context,
                            impl_traits: None,
                        },
                    },
                    raw_attributes: attrs_opt,
                }))
            }
            // Uncomment this when we decide how to handle ImplTraits
            // ImplTrait { decl_id, decl_span, .. } => {
            // TODO: figure out how to use this, likely we don't want to document this directly.
            //
            // This declaration type may make more sense to document as part of another declaration
            // much like how we document method functions for traits or fields on structs.
            //     let impl_trait = decl_engine.get_impl_trait(&decl_ref, decl_span)?;
            //     let item_name = impl_trait.trait_name.suffix;
            //     Ok(Descriptor::Documentable(Document {
            //         module_info: module_info.clone(),
            //         item_header: ItemHeader {
            //             module_info: module_info.clone(),
            //             friendly_name: ty_decl.friendly_name(),
            //             item_name: item_name.clone(),
            //         },
            //         item_body: ItemBody {
            //             module_info,
            //             ty_decl: ty_decl.clone(),
            //             item_name,
            //             code_str: parse::parse_format::<sway_ast::ItemImpl>(
            //                 impl_trait.span.as_str(),
            //             ),
            //             attrs_opt: None, // no attributes field
            //             item_context: ItemContext { context: None },
            //         },
            //         raw_attributes: None,
            //     }))
            // }
            FunctionDecl { decl_id, .. } => {
                let fn_decl = decl_engine.get_function(decl_id);
                if !document_private_items && fn_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = fn_decl.name;
                    let attrs_opt = (!fn_decl.attributes.is_empty())
                        .then(|| fn_decl.attributes.to_html_string());

                    Ok(Descriptor::Documentable(Document {
                        module_info: module_info.clone(),
                        item_header: ItemHeader {
                            module_info: module_info.clone(),
                            friendly_name: ty_decl.friendly_type_name(),
                            item_name: item_name.clone(),
                        },
                        item_body: ItemBody {
                            module_info,
                            ty_decl: ty_decl.clone(),
                            item_name,
                            code_str: trim_fn_body(parse::parse_format::<sway_ast::ItemFn>(
                                fn_decl.span.as_str(),
                            )),
                            attrs_opt: attrs_opt.clone(),
                            item_context: ItemContext {
                                context_opt: None,
                                impl_traits: None,
                            },
                        },
                        raw_attributes: attrs_opt,
                    }))
                }
            }
            ConstantDecl { decl_id, .. } => {
                let const_decl = decl_engine.get_constant(decl_id);
                if !document_private_items && const_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = const_decl.call_path.suffix;
                    let attrs_opt = (!const_decl.attributes.is_empty())
                        .then(|| const_decl.attributes.to_html_string());

                    Ok(Descriptor::Documentable(Document {
                        module_info: module_info.clone(),
                        item_header: ItemHeader {
                            module_info: module_info.clone(),
                            friendly_name: ty_decl.friendly_type_name(),
                            item_name: item_name.clone(),
                        },
                        item_body: ItemBody {
                            module_info,
                            ty_decl: ty_decl.clone(),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemConst>(
                                const_decl.span.as_str(),
                            ),
                            attrs_opt: attrs_opt.clone(),
                            item_context: ItemContext {
                                context_opt: None,
                                impl_traits: None,
                            },
                        },
                        raw_attributes: attrs_opt,
                    }))
                }
            }
            _ => Ok(Descriptor::NonDocumentable),
        }
    }
}
