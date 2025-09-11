//! Determine whether a [Declaration] is documentable.
use crate::{
    doc::{module::ModuleInfo, Document},
    render::{
        item::{
            components::*,
            context::{Context, ContextType, ItemContext},
            documentable_type::DocumentableType,
        },
        util::format::docstring::DocStrings,
    },
};
use anyhow::Result;
use forc_tracing::println_warning;
use sway_core::{
    decl_engine::*,
    language::ty::{self, TyTraitFn, TyTraitInterfaceItem},
    Engines, GenericArgument, TypeInfo,
};
use sway_features::ExperimentalFeatures;
use sway_types::{integer_bits::IntegerBits, Ident};
use swayfmt::parse;

trait RequiredMethods {
    fn to_methods(&self, decl_engine: &DeclEngine) -> Vec<TyTraitFn>;
}

impl RequiredMethods for Vec<DeclRefTraitFn> {
    fn to_methods(&self, decl_engine: &DeclEngine) -> Vec<TyTraitFn> {
        self.iter()
            .map(|decl_ref| decl_engine.get_trait_fn(decl_ref).as_ref().clone())
            .collect()
    }
}

/// Used in deciding whether or not a [Declaration] is documentable.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Descriptor {
    Documentable(Document),
    NonDocumentable,
}

impl Descriptor {
    /// Decides whether a [ty::TyDecl] is [Descriptor::Documentable] and returns a [Document] if so.
    pub(crate) fn from_typed_decl(
        decl_engine: &DeclEngine,
        ty_decl: &ty::TyDecl,
        module_info: ModuleInfo,
        document_private_items: bool,
        experimental: ExperimentalFeatures,
    ) -> Result<Self> {
        const CONTRACT_STORAGE: &str = "Contract Storage";
        match ty_decl {
            ty::TyDecl::StructDecl(ty::StructDecl { decl_id, .. }) => {
                let struct_decl = decl_engine.get_struct(decl_id);
                if !document_private_items && struct_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = struct_decl.call_path.suffix.clone();
                    let attrs_opt = (!struct_decl.attributes.is_empty())
                        .then(|| struct_decl.attributes.to_html_string());
                    let context = (!struct_decl.fields.is_empty()).then_some(Context::new(
                        module_info.clone(),
                        ContextType::StructFields(struct_decl.fields.clone()),
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
                            ty: DocumentableType::Declared(ty_decl.clone()),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemStruct>(
                                struct_decl.span.as_str(),
                                experimental,
                            )?,
                            attrs_opt: attrs_opt.clone(),
                            item_context: ItemContext {
                                context_opt: context,
                                ..Default::default()
                            },
                        },
                        raw_attributes: attrs_opt,
                    }))
                }
            }
            ty::TyDecl::EnumDecl(ty::EnumDecl { decl_id, .. }) => {
                let enum_decl = decl_engine.get_enum(decl_id);
                if !document_private_items && enum_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = enum_decl.call_path.suffix.clone();
                    let attrs_opt = (!enum_decl.attributes.is_empty())
                        .then(|| enum_decl.attributes.to_html_string());
                    let context = (!enum_decl.variants.is_empty()).then_some(Context::new(
                        module_info.clone(),
                        ContextType::EnumVariants(enum_decl.variants.clone()),
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
                            ty: DocumentableType::Declared(ty_decl.clone()),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemEnum>(
                                enum_decl.span.as_str(),
                                experimental,
                            )?,
                            attrs_opt: attrs_opt.clone(),
                            item_context: ItemContext {
                                context_opt: context,
                                ..Default::default()
                            },
                        },
                        raw_attributes: attrs_opt,
                    }))
                }
            }
            ty::TyDecl::TraitDecl(ty::TraitDecl { decl_id, .. }) => {
                let trait_decl = (*decl_engine.get_trait(decl_id)).clone();
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
                                    .filter_map(|item| match item {
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
                            ty: DocumentableType::Declared(ty_decl.clone()),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemTrait>(
                                trait_decl.span.as_str(),
                                experimental,
                            )?,
                            attrs_opt: attrs_opt.clone(),
                            item_context: ItemContext {
                                context_opt: context,
                                ..Default::default()
                            },
                        },
                        raw_attributes: attrs_opt,
                    }))
                }
            }
            ty::TyDecl::AbiDecl(ty::AbiDecl { decl_id, .. }) => {
                let abi_decl = (*decl_engine.get_abi(decl_id)).clone();
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
                        ty: DocumentableType::Declared(ty_decl.clone()),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemAbi>(
                            abi_decl.span.as_str(),
                            experimental,
                        )?,
                        attrs_opt: attrs_opt.clone(),
                        item_context: ItemContext {
                            context_opt: context,
                            ..Default::default()
                        },
                    },
                    raw_attributes: attrs_opt,
                }))
            }
            ty::TyDecl::StorageDecl(ty::StorageDecl { decl_id, .. }) => {
                let storage_decl = decl_engine.get_storage(decl_id);
                let item_name = sway_types::BaseIdent::new_no_trim(
                    sway_types::span::Span::from_string(CONTRACT_STORAGE.to_string()),
                );
                let attrs_opt = (!storage_decl.attributes.is_empty())
                    .then(|| storage_decl.attributes.to_html_string());
                let context = (!storage_decl.fields.is_empty()).then_some(Context::new(
                    module_info.clone(),
                    ContextType::StorageFields(storage_decl.fields.clone()),
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
                        ty: DocumentableType::Declared(ty_decl.clone()),
                        item_name,
                        code_str: parse::parse_format::<sway_ast::ItemStorage>(
                            storage_decl.span.as_str(),
                            experimental,
                        )?,
                        attrs_opt: attrs_opt.clone(),
                        item_context: ItemContext {
                            context_opt: context,
                            ..Default::default()
                        },
                    },
                    raw_attributes: attrs_opt,
                }))
            }
            ty::TyDecl::FunctionDecl(ty::FunctionDecl { decl_id, .. }) => {
                let fn_decl = decl_engine.get_function(decl_id);
                if !document_private_items && fn_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = fn_decl.name.clone();
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
                            ty: DocumentableType::Declared(ty_decl.clone()),
                            item_name,
                            code_str: trim_fn_body(parse::parse_format::<sway_ast::ItemFn>(
                                fn_decl.span.as_str(),
                                experimental,
                            )?),
                            attrs_opt: attrs_opt.clone(),
                            item_context: ItemContext {
                                context_opt: None,
                                ..Default::default()
                            },
                        },
                        raw_attributes: attrs_opt,
                    }))
                }
            }
            ty::TyDecl::ConstantDecl(ty::ConstantDecl { decl_id, .. }) => {
                let const_decl = decl_engine.get_constant(decl_id);
                if !document_private_items && const_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = const_decl.call_path.suffix.clone();
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
                            ty: DocumentableType::Declared(ty_decl.clone()),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemConst>(
                                const_decl.span.as_str(),
                                experimental,
                            )?,
                            attrs_opt: attrs_opt.clone(),
                            item_context: Default::default(),
                        },
                        raw_attributes: attrs_opt,
                    }))
                }
            }
            ty::TyDecl::TypeAliasDecl(ty::TypeAliasDecl { decl_id }) => {
                let type_alias_decl = decl_engine.get_type_alias(decl_id);
                if !document_private_items && type_alias_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = type_alias_decl.name.clone();
                    let attrs_opt = (!type_alias_decl.attributes.is_empty())
                        .then(|| type_alias_decl.attributes.to_html_string());

                    let GenericArgument::Type(t) = &type_alias_decl.ty else {
                        unreachable!()
                    };
                    let code_str = parse::parse_format::<sway_ast::ItemTypeAlias>(
                        &format!(
                            "{}type {} = {};",
                            if type_alias_decl.visibility.is_public() {
                                "pub "
                            } else {
                                ""
                            },
                            type_alias_decl.span.as_str(),
                            t.span.as_str()
                        ),
                        experimental,
                    )?;

                    Ok(Descriptor::Documentable(Document {
                        module_info: module_info.clone(),
                        item_header: ItemHeader {
                            module_info: module_info.clone(),
                            friendly_name: ty_decl.friendly_type_name(),
                            item_name: item_name.clone(),
                        },
                        item_body: ItemBody {
                            module_info,
                            ty: DocumentableType::Declared(ty_decl.clone()),
                            item_name,
                            code_str,
                            attrs_opt: attrs_opt.clone(),
                            item_context: ItemContext {
                                context_opt: None,
                                ..Default::default()
                            },
                        },
                        raw_attributes: attrs_opt,
                    }))
                }
            }
            _ => {
                println_warning(&format!("Non-documentable declaration: {:?}", ty_decl));
                Ok(Descriptor::NonDocumentable)
            }
        }
    }

    /// Decides whether a [TypeInfo] is [Descriptor::Documentable] and returns a [Document] if so.
    pub(crate) fn from_type_info(
        type_info: &TypeInfo,
        engines: &Engines,
        module_info: ModuleInfo,
    ) -> Result<Self> {
        // Only primitive types will result in a documentable item. All other type documentation should come
        // from the a declaration. Since primitive types do not have sway declarations, we can only generate
        // documentation from their implementations.
        let item_name = Ident::new_no_span(format!("{}", engines.help_out(type_info)));
        // Build a fake module info for the primitive type.
        let module_info = ModuleInfo {
            module_prefixes: vec![module_info.project_name().into()],
            attributes: None,
        };
        // TODO: Find a way to add descriptions without hardcoding them.
        let description = match type_info {
            TypeInfo::StringSlice => "string slice",
            TypeInfo::StringArray(_) => "fixed-length string",
            TypeInfo::Boolean => "Boolean true or false",
            TypeInfo::B256 => "256 bits (32 bytes), i.e. a hash",
            TypeInfo::UnsignedInteger(bits) => match bits {
                IntegerBits::Eight => "8-bit unsigned integer",
                IntegerBits::Sixteen => "16-bit unsigned integer",
                IntegerBits::ThirtyTwo => "32-bit unsigned integer",
                IntegerBits::SixtyFour => "64-bit unsigned integer",
                IntegerBits::V256 => "256-bit unsigned integer",
            },
            _ => return Ok(Descriptor::NonDocumentable),
        };
        let attrs_opt = Some(description.to_string());

        match type_info {
            TypeInfo::StringSlice
            | TypeInfo::StringArray(_)
            | TypeInfo::Boolean
            | TypeInfo::B256
            | TypeInfo::UnsignedInteger(_) => Ok(Descriptor::Documentable(Document {
                module_info: module_info.clone(),
                item_header: ItemHeader {
                    module_info: module_info.clone(),
                    friendly_name: "primitive",
                    item_name: item_name.clone(),
                },
                item_body: ItemBody {
                    module_info,
                    ty: DocumentableType::Primitive(type_info.clone()),
                    item_name: item_name.clone(),
                    code_str: item_name.to_string(),
                    attrs_opt: attrs_opt.clone(),
                    item_context: Default::default(),
                },
                raw_attributes: attrs_opt,
            })),
            _ => Ok(Descriptor::NonDocumentable),
        }
    }
}

/// Takes a formatted function signature & body and returns only the signature.
fn trim_fn_body(f: String) -> String {
    match f.find('{') {
        Some(index) => f.split_at(index).0.to_string(),
        None => f,
    }
}
