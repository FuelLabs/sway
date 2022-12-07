//! Determine whether a [Declaration] is documentable.
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
    Documentable {
        // If empty, this is the root.
        module_prefix: Vec<String>,
        // We want _all_ of the TyDeclaration information.
        desc_ty: Box<DescriptorType>,
    },
    NonDocumentable,
}

impl Descriptor {
    pub(crate) fn from_typed_decl(d: &TyDeclaration, module_prefix: Vec<String>) -> Result<Self> {
        use TyDeclaration::*;
        match d {
            StructDeclaration(ref decl_id) => {
                let struct_decl = de_get_struct(decl_id.clone(), &decl_id.span())?;
                Ok(Descriptor::Documentable {
                    module_prefix,
                    desc_ty: Box::new(DescriptorType::Struct(struct_decl)),
                })
            }
            EnumDeclaration(ref decl_id) => {
                let enum_decl = de_get_enum(decl_id.clone(), &decl_id.span())?;
                Ok(Descriptor::Documentable {
                    module_prefix,
                    desc_ty: Box::new(DescriptorType::Enum(enum_decl)),
                })
            }
            TraitDeclaration(ref decl_id) => {
                let trait_decl = de_get_trait(decl_id.clone(), &decl_id.span())?;
                Ok(Descriptor::Documentable {
                    module_prefix,
                    desc_ty: Box::new(DescriptorType::Trait(trait_decl)),
                })
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
                Ok(Descriptor::Documentable {
                    module_prefix,
                    desc_ty: Box::new(DescriptorType::Function(fn_decl)),
                })
            }
            ConstantDeclaration(ref decl_id) => {
                let const_decl = de_get_constant(decl_id.clone(), &decl_id.span())?;
                Ok(Descriptor::Documentable {
                    module_prefix,
                    desc_ty: Box::new(DescriptorType::Const(Box::new(const_decl))),
                })
            }
            _ => Ok(Descriptor::NonDocumentable),
        }
    }
}
