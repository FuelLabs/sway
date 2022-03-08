use crate::utils::function::extract_fn_signature;
use sway_core::{FunctionDeclaration, StructDeclaration, TraitDeclaration, Visibility};

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Library,
    Variable(VariableDetails),
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

impl FunctionDetails {
    pub fn get_return_type_from_signature(&self) -> Option<String> {
        self.signature
            .split("->")
            .nth(1)
            .map(|return_type| return_type.trim().split(' ').take(1).collect())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDetails {
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraitDetails {
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableDetails {
    pub is_mutable: bool,
    pub var_body: VarBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VarBody {
    FunctionCall(String),
    Type(String),
    Other,
}
