use super::*;
use crate::{
    build_config::BuildConfig, error::*, semantic_analysis::ast_node::TypedStructField,
    semantic_analysis::TypedExpression, types::ResolvedType, CallPath, Ident, Rule, Span,
};
use derivative::Derivative;
use std::collections::HashMap;
use std::iter::FromIterator;

use pest::iterators::Pair;
/// Type information without an associated value, used for type inferencing and definition.
#[derive(Derivative)]
#[derivative(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypeInfo<'sc> {
    Unknown,
    Str(u64),
    UnsignedInteger(IntegerBits),
    Enum {
        name: Ident<'sc>,
        variant_types: Vec<TypeId>,
    },
    Struct {
        name: Ident<'sc>,
        fields: Vec<TypedStructField<'sc>>,
    },
    Boolean,
    /// A custom type could be a struct or similar if the name is in scope,
    /// or just a generic parameter if it is not.
    /// At parse time, there is no sense of scope, so this determination is not made
    /// until the semantic analysis stage.
    Custom {
        name: crate::Ident<'sc>,
    },
    /// For the type inference engine to use when a type references another type
    Ref(TypeId),

    Unit,
    /// Represents a type which contains methods to issue a contract call.
    /// The specific contract is identified via the `Ident` within.
    ContractCaller {
        abi_name: CallPath<'sc>,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        address: Box<TypedExpression<'sc>>,
    },
    SelfType,
    Byte,
    B256,
    /// This means that specific type of a number is not yet known. It will be
    /// determined via inference at a later time.
    Numeric,
    Contract,
    // used for recovering from errors in the ast
    ErrorRecovery,
}

impl Default for TypeInfo<'_> {
    fn default() -> Self {
        TypeInfo::Unknown
    }
}

impl<'sc> TypeInfo<'sc> {
    pub(crate) fn parse_from_pair(
        input: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut r#type = input.into_inner();
        Self::parse_from_pair_inner(r#type.next().unwrap(), config)
    }

    pub(crate) fn parse_from_pair_inner(
        input: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut warnings = vec![];
        let mut errors = vec![];
        let input = if let Some(input) = input.clone().into_inner().next() {
            input
        } else {
            input
        };
        ok(
            match input.as_str().trim() {
                "u8" => TypeInfo::UnsignedInteger(IntegerBits::Eight),
                "u16" => TypeInfo::UnsignedInteger(IntegerBits::Sixteen),
                "u32" => TypeInfo::UnsignedInteger(IntegerBits::ThirtyTwo),
                "u64" => TypeInfo::UnsignedInteger(IntegerBits::SixtyFour),
                "bool" => TypeInfo::Boolean,
                "unit" => TypeInfo::Unit,
                "byte" => TypeInfo::Byte,
                "b256" => TypeInfo::B256,
                "Self" | "self" => TypeInfo::SelfType,
                "Contract" => TypeInfo::Contract,
                "()" => TypeInfo::Unit,
                a if a.contains("str[") => check!(
                    parse_str_type(
                        a,
                        Span {
                            span: input.as_span(),
                            path: config.map(|config| config.dir_of_code.clone())
                        }
                    ),
                    return err(warnings, errors),
                    warnings,
                    errors
                ),
                _other => TypeInfo::Custom {
                    name: check!(
                        Ident::parse_from_pair(input, config),
                        return err(warnings, errors),
                        warnings,
                        errors
                    ),
                },
            },
            warnings,
            errors,
        )
    }

    pub(crate) fn friendly_type_str(&self) -> String {
        use TypeInfo::*;
        match self {
            Unknown => "unknown".into(),
            Str(x) => format!("str[{}]", x),
            UnsignedInteger(x) => match x {
                IntegerBits::Eight => "u8",
                IntegerBits::Sixteen => "u16",
                IntegerBits::ThirtyTwo => "u32",
                IntegerBits::SixtyFour => "u64",
            }
            .into(),
            Boolean => "bool".into(),
            Custom { name } => format!("{}", name.primary_name),
            Ref(id) => format!("T{}", id),
            Unit => "()".into(),
            SelfType => "Self".into(),
            Byte => "byte".into(),
            B256 => "b256".into(),
            Numeric => "numeric".into(),
            Contract => "contract".into(),
            ErrorRecovery => "unknown due to error".into(),
            Enum { name, .. } => format!("enum {}", name.primary_name),
            Struct { name, .. } => format!("struct {}", name.primary_name),
            ContractCaller { abi_name, .. } => {
                format!("contract caller {}", abi_name.suffix.primary_name)
            }
        }
    }
}
