use sway_core::language::ty::TyDecl;

pub(crate) trait DocBlockTitle {
    fn as_block_title(&self) -> BlockTitle;
}
/// Represents all of the possible titles
/// belonging to an index or sidebar.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum BlockTitle {
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
}
impl BlockTitle {
    pub(crate) fn as_str(&self) -> &str {
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
        }
    }
    pub(crate) fn item_title_str(&self) -> &str {
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
        }
    }
    pub(crate) fn class_title_str(&self) -> &str {
        match self {
            Self::Modules => "mod",
            Self::Structs => "struct",
            Self::Enums => "enum",
            Self::Traits => "trait",
            Self::Abi => "abi",
            Self::ContractStorage => "storage",
            Self::Constants => "constant",
            Self::Functions => "fn",
            _ => unimplemented!("These titles are unimplemented, and should not be used this way."),
        }
    }
    pub(crate) fn html_title_string(&self) -> String {
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

impl DocBlockTitle for TyDecl {
    fn as_block_title(&self) -> BlockTitle {
        match self {
            TyDecl::StructDecl { .. } => BlockTitle::Structs,
            TyDecl::EnumDecl { .. } => BlockTitle::Enums,
            TyDecl::TraitDecl { .. } => BlockTitle::Traits,
            TyDecl::AbiDecl { .. } => BlockTitle::Abi,
            TyDecl::StorageDecl { .. } => BlockTitle::ContractStorage,
            TyDecl::ConstantDecl { .. } => BlockTitle::Constants,
            TyDecl::FunctionDecl { .. } => BlockTitle::Functions,
            _ => {
                unreachable!("All other TyDecls are non-documentable and will never be matched on")
            }
        }
    }
}
