//! Determine whether a [Declaration] is documentable.
use crate::{
    doc::{Document, ModuleInfo},
    render::{attrsmap_to_html_str, trim_fn_body, ContextType, ItemBody, ItemContext, ItemHeader},
};
use anyhow::Result;
use sway_core::{declaration_engine::*, language::ty::TyDeclaration};
use sway_types::Spanned;

/// Used in deciding whether or not a [Declaration] is documentable.
pub(crate) enum Descriptor<'dir, 'mdl_info> {
    Documentable(Document<'dir, 'mdl_info>),
    NonDocumentable,
}

impl<'dir> Descriptor<'_, 'dir> {
    /// Decides whether a [TyDeclaration] is [Descriptor::Documentable].
    pub(crate) fn from_typed_decl(
        ty_decl: &TyDeclaration,
        module_info: ModuleInfo,
        document_private_items: bool,
    ) -> Result<Self> {
        const CONTRACT_STORAGE: &str = "Contract Storage";

        use swayfmt::parse;
        use TyDeclaration::*;
        match ty_decl {
            StructDeclaration(ref decl_id) => {
                let struct_decl = de_get_struct(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && struct_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = struct_decl.name.as_str();
                    let attrs_opt = (!struct_decl.attributes.is_empty())
                        .then(|| attrsmap_to_html_str(&struct_decl.attributes));
                    let context = (!struct_decl.fields.is_empty())
                        .then_some(ContextType::StructFields(struct_decl.fields));

                    Ok(Descriptor::Documentable(Document {
                        module_info: &module_info,
                        item_header: ItemHeader {
                            module_info: &module_info,
                            friendly_name: ty_decl.friendly_name(),
                            item_name,
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
                    }))
                }
            }
            EnumDeclaration(ref decl_id) => {
                let enum_decl = de_get_enum(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && enum_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = enum_decl.name.as_str();
                    let attrs_opt = (!enum_decl.attributes.is_empty())
                        .then(|| attrsmap_to_html_str(&enum_decl.attributes));
                    let context = (!enum_decl.variants.is_empty())
                        .then_some(ContextType::EnumVariants(enum_decl.variants));

                    Ok(Descriptor::Documentable(Document {
                        module_info: &module_info,
                        item_header: ItemHeader {
                            module_info: &module_info,
                            friendly_name: ty_decl.friendly_name(),
                            item_name,
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
                    }))
                }
            }
            TraitDeclaration(ref decl_id) => {
                let trait_decl = de_get_trait(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && trait_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = trait_decl.name.as_str();
                    let attrs_opt = (!trait_decl.attributes.is_empty())
                        .then(|| attrsmap_to_html_str(&trait_decl.attributes));
                    let context = (!trait_decl.interface_surface.is_empty())
                        .then_some(ContextType::RequiredMethods(trait_decl.interface_surface));

                    Ok(Descriptor::Documentable(Document {
                        module_info: &module_info,
                        item_header: ItemHeader {
                            module_info: &module_info,
                            friendly_name: ty_decl.friendly_name(),
                            item_name,
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
                    }))
                }
            }
            AbiDeclaration(ref decl_id) => {
                let abi_decl = de_get_abi(decl_id.clone(), &decl_id.span())?;
                let item_name = abi_decl.name.as_str();
                let attrs_opt = (!abi_decl.attributes.is_empty())
                    .then(|| attrsmap_to_html_str(&abi_decl.attributes));
                let context = (!abi_decl.interface_surface.is_empty())
                    .then_some(ContextType::RequiredMethods(abi_decl.interface_surface));

                Ok(Descriptor::Documentable(Document {
                    module_info: &module_info,
                    item_header: ItemHeader {
                        module_info: &module_info,
                        friendly_name: ty_decl.friendly_name(),
                        item_name,
                    },
                    item_body: ItemBody {
                        module_info,
                        ty_decl: ty_decl.clone(),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemAbi>(abi_decl.span.as_str()),
                        attrs_opt,
                        item_context: ItemContext { context },
                    },
                }))
            }
            StorageDeclaration(ref decl_id) => {
                let storage_decl = de_get_storage(decl_id.clone(), &decl_id.span())?;
                let item_name = CONTRACT_STORAGE;
                let attrs_opt = (!storage_decl.attributes.is_empty())
                    .then(|| attrsmap_to_html_str(&storage_decl.attributes));
                let context = (!storage_decl.fields.is_empty())
                    .then_some(ContextType::StorageFields(storage_decl.fields));

                Ok(Descriptor::Documentable(Document {
                    module_info: &module_info,
                    item_header: ItemHeader {
                        module_info: &module_info,
                        friendly_name: ty_decl.friendly_name(),
                        item_name,
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
                }))
            }
            ImplTrait(ref decl_id) => {
                // TODO: figure out how to use this, likely we don't want to document this directly.
                //
                // This declaration type may make more sense to document as part of another declaration
                // much like how we document method functions for traits or fields on structs.
                let impl_trait = de_get_impl_trait(decl_id.clone(), &decl_id.span())?;
                let item_name = impl_trait.trait_name.suffix.as_str();

                Ok(Descriptor::Documentable(Document {
                    module_info: &module_info,
                    item_header: ItemHeader {
                        module_info: &module_info,
                        friendly_name: ty_decl.friendly_name(),
                        item_name,
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
                }))
            }
            FunctionDeclaration(ref decl_id) => {
                let fn_decl = de_get_function(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && fn_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = fn_decl.name.as_str();
                    let attrs_opt = (!fn_decl.attributes.is_empty())
                        .then(|| attrsmap_to_html_str(&fn_decl.attributes));

                    Ok(Descriptor::Documentable(Document {
                        module_info: &module_info,
                        item_header: ItemHeader {
                            module_info: &module_info,
                            friendly_name: ty_decl.friendly_name(),
                            item_name,
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
                    }))
                }
            }
            ConstantDeclaration(ref decl_id) => {
                let const_decl = de_get_constant(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && const_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = const_decl.name.as_str();
                    let attrs_opt = (!const_decl.attributes.is_empty())
                        .then(|| attrsmap_to_html_str(&const_decl.attributes));

                    Ok(Descriptor::Documentable(Document {
                        module_info: &module_info,
                        item_header: ItemHeader {
                            module_info: &module_info,
                            friendly_name: ty_decl.friendly_name(),
                            item_name,
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
                    }))
                }
            }
            _ => Ok(Descriptor::NonDocumentable),
        }
    }
}
