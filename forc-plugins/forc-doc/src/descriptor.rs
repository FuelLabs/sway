//! Determine whether a [Declaration] is documentable.
use crate::{
    doc::Document,
    render::{
        attrsmap_to_html_string, struct_field_section, ItemBody, ItemContext, ItemHeader,
        MainContent,
    },
};
use anyhow::Result;
use sway_core::{
    declaration_engine::*,
    language::ty::{
        TyAbiDeclaration, TyConstantDeclaration, TyDeclaration, TyEnumDeclaration,
        TyFunctionDeclaration, TyImplTrait, TyStorageDeclaration, TyStructDeclaration,
        TyTraitDeclaration,
    },
};
use sway_types::Spanned;

// TODO: See if there's a way we can use the TyDeclarations directly
//
/// The type of [TyDeclaration] documented by the [Descriptor].
#[derive(Debug)]
pub(crate) enum DescriptorType {
    Struct(TyStructDeclaration),
    Enum(TyEnumDeclaration),
    Trait(TyTraitDeclaration),
    Abi(TyAbiDeclaration),
    Storage(TyStorageDeclaration),
    ImplTraitDesc(TyImplTrait),
    Function(TyFunctionDeclaration),
    Const(Box<TyConstantDeclaration>),
}

impl DescriptorType {
    /// Converts the [DescriptorType] to a `&str` name for HTML file name creation.
    pub fn as_str(&self) -> &'static str {
        use DescriptorType::*;
        match self {
            Struct(_) => "struct",
            Enum(_) => "enum",
            Trait(_) => "trait",
            Abi(_) => "abi",
            Storage(_) => "storage",
            ImplTraitDesc(_) => "impl_trait",
            Function(_) => "fn",
            Const(_) => "constant",
        }
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
        ty_decl: &TyDeclaration,
        module_prefix: Vec<String>,
        document_private_items: bool,
    ) -> Result<Self> {
        let module_depth = module_prefix.len();
        let module = *module_prefix.last().unwrap(); // There will always be at least the project name

        use swayfmt::parse;
        use TyDeclaration::*;
        match ty_decl {
            StructDeclaration(ref decl_id) => {
                let struct_decl = de_get_struct(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && struct_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    let item_name = struct_decl.name.as_str().to_string();
                    let item_header = ItemHeader {
                        module_depth,
                        module,
                        ty_decl: ty_decl.clone(),
                        item_name: item_name.clone(),
                    };
                    let item_body = ItemBody {
                        main_content: MainContent {
                            module_depth,
                            ty_decl: ty_decl.clone(),
                            item_name,
                            code_str: parse::parse_format::<sway_ast::ItemStruct>(
                                struct_decl.span.as_str(),
                            ),
                            attrs_str: attrsmap_to_html_string(&struct_decl.attributes),
                        },
                        item_context: ItemContext(struct_field_section(struct_decl.fields.clone())),
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
                    Ok(Descriptor::Documentable {
                        module_prefix,
                        desc_ty: Box::new(DescriptorType::Enum(enum_decl)),
                    })
                }
            }
            TraitDeclaration(ref decl_id) => {
                let trait_decl = de_get_trait(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && trait_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    Ok(Descriptor::Documentable {
                        module_prefix,
                        desc_ty: Box::new(DescriptorType::Trait(trait_decl)),
                    })
                }
            }
            AbiDeclaration(ref decl_id) => {
                let abi_decl = de_get_abi(decl_id.clone(), &decl_id.span())?;
                Ok(Descriptor::Documentable {
                    module_prefix,
                    desc_ty: Box::new(DescriptorType::Abi(abi_decl)),
                })
            }
            StorageDeclaration(ref decl_id) => {
                let storage_decl = de_get_storage(decl_id.clone(), &decl_id.span())?;
                Ok(Descriptor::Documentable {
                    module_prefix,
                    desc_ty: Box::new(DescriptorType::Storage(storage_decl)),
                })
            }
            ImplTrait(ref decl_id) => {
                let impl_trait = de_get_impl_trait(decl_id.clone(), &decl_id.span())?;
                Ok(Descriptor::Documentable {
                    module_prefix,
                    desc_ty: Box::new(DescriptorType::ImplTraitDesc(impl_trait)),
                })
            }
            FunctionDeclaration(ref decl_id) => {
                let fn_decl = de_get_function(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && fn_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    Ok(Descriptor::Documentable {
                        module_prefix,
                        desc_ty: Box::new(DescriptorType::Function(fn_decl)),
                    })
                }
            }
            ConstantDeclaration(ref decl_id) => {
                let const_decl = de_get_constant(decl_id.clone(), &decl_id.span())?;
                if !document_private_items && const_decl.visibility.is_private() {
                    Ok(Descriptor::NonDocumentable)
                } else {
                    Ok(Descriptor::Documentable {
                        module_prefix,
                        desc_ty: Box::new(DescriptorType::Const(Box::new(const_decl))),
                    })
                }
            }
            _ => Ok(Descriptor::NonDocumentable),
        }
    }
}
