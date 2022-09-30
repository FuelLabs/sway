use sway_core::{declaration_engine::*, AbiName, Declaration, TypeInfo, TypedDeclaration};
use sway_types::{Ident, Spanned};

#[derive(Eq, PartialEq, Ord, PartialOrd, Debug)]
pub(crate) enum DescriptorType {
    Struct,
    Enum,
    Trait,
    Abi,
    Storage,
    ImplSelfDesc,
    ImplTraitDesc,
    Function,
    Const,
}
impl DescriptorType {
    pub fn to_name(&self) -> &'static str {
        use DescriptorType::*;
        match self {
            Struct => "struct",
            Enum => "enum",
            Trait => "trait",
            Abi => "abi",
            Storage => "storage",
            ImplSelfDesc => "impl_self",
            ImplTraitDesc => "impl_trait",
            Function => "function",
            Const => "const",
        }
    }
}

#[derive(Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum Descriptor {
    Documentable {
        ty: DescriptorType,
        name: Option<Ident>,
    },
    NonDocumentable,
}
impl Descriptor {
    pub fn to_file_name(&self) -> Option<String> {
        use Descriptor::*;
        match self {
            NonDocumentable => None,
            Documentable { ty, name } => {
                let name_str = match name {
                    Some(name) => name.as_str(),
                    None => ty.to_name(),
                };
                Some(format!("{}.{}.html", ty.to_name(), name_str))
            }
        }
    }
}
impl From<&Declaration> for Descriptor {
    fn from(d: &Declaration) -> Self {
        use Declaration::*;
        use DescriptorType::*;
        match d {
            StructDeclaration(ref decl) => Descriptor::Documentable {
                ty: Struct,
                name: Some(decl.name.clone()),
            },
            EnumDeclaration(ref decl) => Descriptor::Documentable {
                ty: Enum,
                name: Some(decl.name.clone()),
            },
            TraitDeclaration(ref decl) => Descriptor::Documentable {
                ty: Trait,
                name: Some(decl.name.clone()),
            },
            AbiDeclaration(ref decl) => Descriptor::Documentable {
                ty: Abi,
                name: Some(decl.name.clone()),
            },
            StorageDeclaration(_) => Descriptor::Documentable {
                ty: Storage,
                name: None, // no ident
            },
            ImplSelf(ref decl) => Descriptor::Documentable {
                ty: ImplSelfDesc,
                // possible ident
                name: match decl.type_implementing_for {
                    TypeInfo::UnknownGeneric { ref name } => Some(name.clone()),
                    TypeInfo::Enum {
                        ref name,
                        type_parameters: _,
                        variant_types: _,
                    } => Some(name.clone()),
                    TypeInfo::Struct {
                        ref name,
                        type_parameters: _,
                        fields: _,
                    } => Some(name.clone()),
                    TypeInfo::ContractCaller {
                        ref abi_name,
                        address: _,
                    } => match abi_name {
                        AbiName::Known(name) => Some(name.suffix.clone()),
                        AbiName::Deferred => None,
                    },
                    TypeInfo::Custom {
                        ref name,
                        type_arguments: _,
                    } => Some(name.clone()),
                    _ => None,
                },
            },
            ImplTrait(ref decl) => Descriptor::Documentable {
                ty: ImplTraitDesc,
                name: Some(decl.trait_name.suffix.clone()),
            },
            FunctionDeclaration(ref decl) => Descriptor::Documentable {
                ty: Function,
                name: Some(decl.name.clone()),
            },
            ConstantDeclaration(ref decl) => Descriptor::Documentable {
                ty: Const,
                name: Some(decl.name.clone()),
            },
            _ => Descriptor::NonDocumentable,
        }
    }
}
impl From<&TypedDeclaration> for Descriptor {
    fn from(d: &TypedDeclaration) -> Self {
        use DescriptorType::*;
        use TypedDeclaration::*;
        match d {
            StructDeclaration(ref decl) => Descriptor::Documentable {
                ty: Struct,
                name: Some(de_get_struct(decl.clone(), &decl.span()).unwrap().name),
            },
            EnumDeclaration(ref decl) => Descriptor::Documentable {
                ty: Enum,
                name: Some(de_get_enum(decl.clone(), &decl.span()).unwrap().name),
            },
            TraitDeclaration(ref decl) => Descriptor::Documentable {
                ty: Trait,
                name: Some(de_get_trait(decl.clone(), &decl.span()).unwrap().name),
            },
            AbiDeclaration(ref decl) => Descriptor::Documentable {
                ty: Abi,
                name: Some(de_get_abi(decl.clone(), &decl.span()).unwrap().name),
            },
            StorageDeclaration(_) => Descriptor::Documentable {
                ty: Storage,
                name: None,
            },
            ImplTrait(ref decl) => Descriptor::Documentable {
                ty: ImplTraitDesc,
                name: Some(
                    de_get_impl_trait(decl.clone(), &decl.span())
                        .unwrap()
                        .trait_name
                        .suffix,
                ),
            },
            FunctionDeclaration(ref decl) => Descriptor::Documentable {
                ty: Function,
                name: Some(de_get_function(decl.clone(), &decl.span()).unwrap().name),
            },
            ConstantDeclaration(ref decl) => Descriptor::Documentable {
                ty: Const,
                name: Some(de_get_constant(decl.clone(), &decl.span()).unwrap().name),
            },
            _ => Descriptor::NonDocumentable,
        }
    }
}
