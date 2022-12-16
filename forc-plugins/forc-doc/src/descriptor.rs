//! Determine whether a [Declaration] is documentable.
use crate::{
    doc::Document,
    render::{attrsmap_to_html_string, ItemBody, ItemHeader},
};
use anyhow::Result;
use sway_core::{declaration_engine::*, language::ty::TyDeclaration};
use sway_types::Spanned;

/// Used in deciding whether or not a [Declaration] is documentable.
pub(crate) enum Descriptor {
    Documentable(Document),
    NonDocumentable,
}

impl Descriptor {
    /// Decides whether a [TyDeclaration] is [Descriptor::Documentable].
    pub(crate) fn from_typed_decl(
        ty_decl: &TyDeclaration,
        module_prefix: Vec<String>,
        document_private_items: bool,
    ) -> Result<Self> {
        const CONTRACT_STORAGE: &'static str = "Contract Storage";
        let module_depth = module_prefix.len();
        let module = module_prefix.last().unwrap().to_owned(); // There will always be at least the project name

        use swayfmt::parse;
        use TyDeclaration::*;
        match ty_decl {
            StructDeclaration(ref decl_id) => {
                let struct_decl = de_get_struct(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && struct_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = struct_decl.name.as_str().to_string();
                    let attrs_opt = if !struct_decl.attributes.is_empty() {
                        Some(attrsmap_to_html_string(&struct_decl.attributes))
                    } else {
                        None
                    };
                    let item_header = ItemHeader {
                        module_depth,
                        module,
                        friendly_name: ty_decl.friendly_name().to_string(),
                        item_name: item_name.clone(),
                    };
                    let item_body = ItemBody {
                        module_depth,
                        ty_decl: ty_decl.clone(),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemStruct>(
                            struct_decl.span.as_str(),
                        ),
                        attrs_opt,
                    };
                    Ok(Descriptor::Documentable(Document {
                        module_prefix,
                        item_header,
                        item_body,
                    }))
                }
            }
            EnumDeclaration(ref decl_id) => {
                let enum_decl = de_get_enum(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && enum_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = enum_decl.name.as_str().to_string();
                    let attrs_opt = if !enum_decl.attributes.is_empty() {
                        Some(attrsmap_to_html_string(&enum_decl.attributes))
                    } else {
                        None
                    };
                    let item_header = ItemHeader {
                        module_depth,
                        module,
                        friendly_name: ty_decl.friendly_name().to_string(),
                        item_name: item_name.clone(),
                    };
                    let item_body = ItemBody {
                        module_depth,
                        ty_decl: ty_decl.clone(),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemEnum>(
                            enum_decl.span.as_str(),
                        ),
                        attrs_opt,
                    };
                    Ok(Descriptor::Documentable(Document {
                        module_prefix,
                        item_header,
                        item_body,
                    }))
                }
            }
            TraitDeclaration(ref decl_id) => {
                let trait_decl = de_get_trait(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && trait_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = trait_decl.name.as_str().to_string();
                    let attrs_opt = if !trait_decl.attributes.is_empty() {
                        Some(attrsmap_to_html_string(&trait_decl.attributes))
                    } else {
                        None
                    };
                    let item_header = ItemHeader {
                        module_depth,
                        module,
                        friendly_name: ty_decl.friendly_name().to_string(),
                        item_name: item_name.clone(),
                    };
                    let item_body = ItemBody {
                        module_depth,
                        ty_decl: ty_decl.clone(),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemTrait>(
                            trait_decl.span.as_str(),
                        ),
                        attrs_opt,
                    };
                    Ok(Descriptor::Documentable(Document {
                        module_prefix,
                        item_header,
                        item_body,
                    }))
                }
            }
            AbiDeclaration(ref decl_id) => {
                let abi_decl = de_get_abi(decl_id.clone(), &decl_id.span())?;
                let item_name = abi_decl.name.as_str().to_string();
                let attrs_opt = if !abi_decl.attributes.is_empty() {
                    Some(attrsmap_to_html_string(&abi_decl.attributes))
                } else {
                    None
                };
                let item_header = ItemHeader {
                    module_depth,
                    module,
                    friendly_name: ty_decl.friendly_name().to_string(),
                    item_name: item_name.clone(),
                };
                let item_body = ItemBody {
                    module_depth,
                    ty_decl: ty_decl.clone(),
                    item_name,
                    code_str: parse::parse_format::<sway_ast::ItemAbi>(abi_decl.span.as_str()),
                    attrs_opt,
                };
                Ok(Descriptor::Documentable(Document {
                    module_prefix,
                    item_header,
                    item_body,
                }))
            }
            StorageDeclaration(ref decl_id) => {
                let storage_decl = de_get_storage(decl_id.clone(), &decl_id.span())?;
                let item_name = CONTRACT_STORAGE.to_string();
                let attrs_opt = if !storage_decl.attributes.is_empty() {
                    Some(attrsmap_to_html_string(&storage_decl.attributes))
                } else {
                    None
                };
                let item_header = ItemHeader {
                    module_depth,
                    module,
                    friendly_name: ty_decl.friendly_name().to_string(),
                    item_name: item_name.clone(),
                };
                let item_body = ItemBody {
                    module_depth,
                    ty_decl: ty_decl.clone(),
                    item_name,
                    code_str: parse::parse_format::<sway_ast::ItemStorage>(
                        storage_decl.span.as_str(),
                    ),
                    attrs_opt,
                };
                Ok(Descriptor::Documentable(Document {
                    module_prefix,
                    item_header,
                    item_body,
                }))
            }
            ImplTrait(ref decl_id) => {
                // TODO: figure out how to use this, likely we don't want to document this directly.
                let impl_trait = de_get_impl_trait(decl_id.clone(), &decl_id.span())?;
                let item_name = impl_trait.trait_name.suffix.as_str().to_string();
                let item_header = ItemHeader {
                    module_depth,
                    module,
                    friendly_name: ty_decl.friendly_name().to_string(),
                    item_name: item_name.clone(),
                };
                let item_body = ItemBody {
                    module_depth,
                    ty_decl: ty_decl.clone(),
                    item_name,
                    code_str: parse::parse_format::<sway_ast::ItemImpl>(impl_trait.span.as_str()),
                    attrs_opt: None, // no attributes field
                };
                Ok(Descriptor::Documentable(Document {
                    module_prefix,
                    item_header,
                    item_body,
                }))
            }
            FunctionDeclaration(ref decl_id) => {
                let fn_decl = de_get_function(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && fn_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = fn_decl.name.as_str().to_string();
                    let attrs_opt = if !fn_decl.attributes.is_empty() {
                        Some(attrsmap_to_html_string(&fn_decl.attributes))
                    } else {
                        None
                    };
                    let item_header = ItemHeader {
                        module_depth,
                        module,
                        friendly_name: ty_decl.friendly_name().to_string(),
                        item_name: item_name.clone(),
                    };
                    let item_body = ItemBody {
                        module_depth,
                        ty_decl: ty_decl.clone(),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemFn>(fn_decl.span.as_str()),
                        attrs_opt,
                    };
                    Ok(Descriptor::Documentable(Document {
                        module_prefix,
                        item_header,
                        item_body,
                    }))
                }
            }
            ConstantDeclaration(ref decl_id) => {
                let const_decl = de_get_constant(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && const_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = const_decl.name.as_str().to_string();
                    let attrs_opt = if !const_decl.attributes.is_empty() {
                        Some(attrsmap_to_html_string(&const_decl.attributes))
                    } else {
                        None
                    };
                    let item_header = ItemHeader {
                        module_depth,
                        module,
                        friendly_name: ty_decl.friendly_name().to_string(),
                        item_name: item_name.clone(),
                    };
                    let item_body = ItemBody {
                        module_depth,
                        ty_decl: ty_decl.clone(),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemConst>(
                            const_decl.span.as_str(),
                        ),
                        attrs_opt,
                    };
                    Ok(Descriptor::Documentable(Document {
                        module_prefix,
                        item_header,
                        item_body,
                    }))
                }
            }
            _ => Ok(Descriptor::NonDocumentable),
        }
    }
}
