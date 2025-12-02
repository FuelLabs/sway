use sway_core::{language::ty::TyDecl, TypeInfo};

pub trait DocBlock {
    /// Returns the title of the block that the user will see.
    fn title(&self) -> BlockTitle;
    /// Returns the name of the block that will be used in the html and css.
    fn name(&self) -> &str;
}
/// Represents all of the possible titles
/// belonging to an index or sidebar.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum BlockTitle {
    Modules,
    Structs,
    Enums,
    Traits,
    Abi,
    ContractStorage,
    Constants,
    Functions,
    Fields,
    Variants,
    RequiredMethods,
    ImplMethods,
    ImplTraits,
    Primitives,
}

impl BlockTitle {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Modules => "Modules",
            Self::Structs => "Structs",
            Self::Enums => "Enums",
            Self::Traits => "Traits",
            Self::Abi => "Abi",
            Self::ContractStorage => "Contract Storage",
            Self::Constants => "Constants",
            Self::Functions => "Functions",
            Self::Fields => "Fields",
            Self::Variants => "Variants",
            Self::RequiredMethods => "Required Methods",
            Self::ImplMethods => "Methods",
            Self::ImplTraits => "Trait Implementations",
            Self::Primitives => "Primitives",
        }
    }
    pub fn item_title_str(&self) -> &str {
        match self {
            Self::Modules => "Module",
            Self::Structs => "Struct",
            Self::Enums => "Enum",
            Self::Traits => "Trait",
            Self::Abi => "Abi",
            Self::ContractStorage => "Contract Storage",
            Self::Constants => "Constant",
            Self::Functions => "Function",
            Self::Fields => "Fields",
            Self::Variants => "Variants",
            Self::RequiredMethods => "Required Methods",
            Self::ImplMethods => "Methods",
            Self::ImplTraits => "Trait Implementations",
            Self::Primitives => "Primitive",
        }
    }
    pub fn class_title_str(&self) -> &str {
        match self {
            Self::Modules => "mod",
            Self::Structs => "struct",
            Self::Enums => "enum",
            Self::Traits => "trait",
            Self::Abi => "abi",
            Self::ContractStorage => "storage",
            Self::Constants => "constant",
            Self::Functions => "fn",
            Self::Primitives => "primitive",
            _ => unimplemented!(
                "BlockTitle {:?} is unimplemented, and should not be used this way.",
                self
            ),
        }
    }
    pub fn html_title_string(&self) -> String {
        if self.as_str().contains(' ') {
            self.as_str()
                .to_lowercase()
                .split_whitespace()
                .collect::<Vec<&str>>()
                .join("-")
        } else {
            self.as_str().to_lowercase()
        }
    }
}

impl DocBlock for TyDecl {
    fn title(&self) -> BlockTitle {
        match self {
            TyDecl::StructDecl { .. } => BlockTitle::Structs,
            TyDecl::EnumDecl { .. } => BlockTitle::Enums,
            TyDecl::TraitDecl { .. } => BlockTitle::Traits,
            TyDecl::AbiDecl { .. } => BlockTitle::Abi,
            TyDecl::StorageDecl { .. } => BlockTitle::ContractStorage,
            TyDecl::ConstantDecl { .. } => BlockTitle::Constants,
            TyDecl::FunctionDecl { .. } => BlockTitle::Functions,
            _ => {
                unreachable!(
                    "TyDecls {:?} is non-documentable and should never be matched on.",
                    self
                )
            }
        }
    }

    fn name(&self) -> &str {
        match self {
            TyDecl::StructDecl(_) => "struct",
            TyDecl::EnumDecl(_) => "enum",
            TyDecl::TraitDecl(_) => "trait",
            TyDecl::AbiDecl(_) => "abi",
            TyDecl::StorageDecl(_) => "contract_storage",
            TyDecl::ImplSelfOrTrait(_) => "impl_trait",
            TyDecl::FunctionDecl(_) => "fn",
            TyDecl::ConstantDecl(_) => "constant",
            TyDecl::TypeAliasDecl(_) => "type_alias",
            _ => {
                unreachable!(
                    "TyDecl {:?} is non-documentable and should never be matched on.",
                    self
                )
            }
        }
    }
}

impl DocBlock for TypeInfo {
    fn title(&self) -> BlockTitle {
        match self {
            sway_core::TypeInfo::StringSlice
            | sway_core::TypeInfo::StringArray(_)
            | sway_core::TypeInfo::Boolean
            | sway_core::TypeInfo::B256
            | sway_core::TypeInfo::UnsignedInteger(_) => BlockTitle::Primitives,
            _ => {
                unimplemented!(
                    "TypeInfo {:?} is non-documentable and should never be matched on.",
                    self
                )
            }
        }
    }

    fn name(&self) -> &str {
        match self {
            sway_core::TypeInfo::StringSlice
            | sway_core::TypeInfo::StringArray(_)
            | sway_core::TypeInfo::Boolean
            | sway_core::TypeInfo::B256
            | sway_core::TypeInfo::UnsignedInteger(_) => "primitive",
            _ => {
                unimplemented!(
                    "TypeInfo {:?} is non-documentable and should never be matched on.",
                    self
                )
            }
        }
    }
}
