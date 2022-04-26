use crate::utils::function::extract_fn_signature;
use sway_core::{
    ConstantDeclaration, EnumDeclaration, StructDeclaration, TraitDeclaration, Visibility,
};
use sway_types::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Library,

    VariableDeclaration(VariableDetails),
    FunctionDeclaration(FunctionDetails),
    TraitDeclaration(TraitDetails),
    StructDeclaration(StructDetails),
    EnumDeclaration(EnumDetails),
    Reassignment,
    ImplTrait,
    ImplSelf, //Unused
    AbiDeclaration,
    ConstantDeclaration(ConstDetails),
    StorageDeclaration, //Unused
    TraitFunction,
    EnumVariant,

    Literal, //Unused
    FunctionApplication,
    LazyOperator, //Unused
    VariableExpression,
    Tuple,      //Unused
    TupleIndex, //Unused
    Array,      //Unused
    StructExpression,
    IfExp,         //Unused
    MatchExp,      //Unused
    AsmExpression, //Unused
    MethodApplication,
    SubfieldExpression, //Unused
    DelineatedPath,
    AbiCast,
    ArrayIndex,                 //Unused
    DelayedMatchTypeResolution, //Unused
    StorageAccess,              //Unused
    IfLet,                      //Unused
    SizeOfVal,                  //Unused
    BuiltinGetTypeProperty,     //Unused

    Unknown,
}

pub fn get_function_details(span: &Span, visibility: Visibility) -> FunctionDetails {
    FunctionDetails {
        signature: extract_fn_signature(span),
        visibility,
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

pub fn get_enum_details(enum_dec: &EnumDeclaration) -> EnumDetails {
    EnumDetails {
        visibility: enum_dec.visibility,
    }
}

pub fn get_const_details(const_dec: &ConstantDeclaration) -> ConstDetails {
    ConstDetails {
        visibility: const_dec.visibility,
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
pub struct EnumDetails {
    pub visibility: Visibility,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstDetails {
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
