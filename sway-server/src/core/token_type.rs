use crate::utils::function::extract_fn_signature;
use core_lang::{FunctionDeclaration, StructDeclaration, TraitDeclaration, Visibility};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Library,
    Variable,
    FunctionDeclaration(FunctionDetails),
    FunctionApplication,
    Reassignment,
    Enum,
    Trait(TraitDetails),
    Struct(StructDetails),
}

pub fn get_function_details(func_dec: &FunctionDeclaration) -> FunctionDetails {
    FunctionDetails {
        signature: extract_fn_signature(func_dec),
        visibility: func_dec.visibility,
    }
}

pub fn get_struct_details(struct_dec: &StructDeclaration) -> StructDetails {
    StructDetails {
        visibility: struct_dec.visibility,
    }
}

pub fn get_trait_details(trait_dec: &TraitDeclaration) -> TraitDetails {
    TraitDetails {
        visibility: trait_dec.visibility,
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDetails {
    pub signature: String,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDetails {
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitDetails {
    pub visibility: Visibility,
}
