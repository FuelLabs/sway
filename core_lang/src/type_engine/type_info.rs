use super::*;
use crate::{
    build_config::BuildConfig,
    parse_tree::OwnedCallPath,
    semantic_analysis::ast_node::{OwnedTypedEnumVariant, OwnedTypedStructField},
    Rule, Span,
};
use derivative::Derivative;

use pest::iterators::Pair;
/// Type information without an associated value, used for type inferencing and definition.
// TODO use idents instead of Strings when we have arena spans
#[derive(Derivative)]
#[derivative(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TypeInfo {
    Unknown,
    Str(u64),
    UnsignedInteger(IntegerBits),
    Enum {
        name: String,
        variant_types: Vec<OwnedTypedEnumVariant>,
    },
    Struct {
        name: String,
        fields: Vec<OwnedTypedStructField>,
    },
    Boolean,
    /// For the type inference engine to use when a type references another type
    Ref(TypeId),

    Unit,
    /// Represents a type which contains methods to issue a contract call.
    /// The specific contract is identified via the `Ident` within.
    ContractCaller {
        abi_name: OwnedCallPath,
        // this is raw source code to be evaluated later.
        address: String,
        // TODO(static span): the above String should be a TypedExpression
        //        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        //        address: Box<TypedExpression<'sc>>,
    },
    /// A custom type could be a struct or similar if the name is in scope,
    /// or just a generic parameter if it is not.
    /// At parse time, there is no sense of scope, so this determination is not made
    /// until the semantic analysis stage.
    Custom {
        name: String,
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

impl Default for TypeInfo {
    fn default() -> Self {
        TypeInfo::Unknown
    }
}

impl TypeInfo {
    pub(crate) fn parse_from_pair<'sc>(
        input: Pair<'sc, Rule>,
        config: Option<&BuildConfig>,
    ) -> CompileResult<'sc, Self> {
        let mut r#type = input.into_inner();
        Self::parse_from_pair_inner(r#type.next().unwrap(), config)
    }

    pub(crate) fn parse_from_pair_inner<'sc>(
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
                    name: input.as_str().trim().to_string(),
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
            Custom { name } => name.into(),
            Ref(id) => format!("T{}", id),
            Unit => "()".into(),
            SelfType => "Self".into(),
            Byte => "byte".into(),
            B256 => "b256".into(),
            Numeric => "numeric".into(),
            Contract => "contract".into(),
            ErrorRecovery => "unknown due to error".into(),
            Enum { name, .. } => format!("enum {}", name),
            Struct { name, .. } => format!("struct {}", name),
            ContractCaller { abi_name, .. } => {
                format!("contract caller {}", abi_name.suffix)
            }
        }
    }

    /// maps a type to a name that is used when constructing function selectors
    pub(crate) fn to_selector_name<'sc>(
        &self,
        error_msg_span: &Span<'sc>,
    ) -> CompileResult<'sc, String> {
        use TypeInfo::*;
        let name = match self {
            Str(len) => format!("str[{}]", len),
            UnsignedInteger(bits) => {
                use IntegerBits::*;
                match bits {
                    Eight => "u8",
                    Sixteen => "u16",
                    ThirtyTwo => "u32",
                    SixtyFour => "u64",
                }
                .into()
            }
            Boolean => "bool".into(),

            Unit => "unit".into(),
            Byte => "byte".into(),
            B256 => "b256".into(),
            Struct { fields, .. } => {
                let field_names = {
                    let names = fields
                        .iter()
                        .map(|OwnedTypedStructField { r#type, .. }| {
                            resolve_type(*r#type, error_msg_span)
                                .expect("unreachable?")
                                .to_selector_name(error_msg_span)
                        })
                        .collect::<Vec<CompileResult<String>>>();
                    let mut buf = vec![];
                    for name in names {
                        match name.value {
                            Some(value) => buf.push(value),
                            None => return name,
                        }
                    }
                    buf
                };

                format!("s({})", field_names.join(","))
            }
            Enum { variant_types, .. } => {
                let variant_names = {
                    let names = variant_types
                        .iter()
                        .map(|ty| {
                            let ty = match resolve_type(ty.r#type, error_msg_span) {
                                Err(e) => return err(vec![], vec![e.into()]),
                                Ok(ty) => ty,
                            };
                            ty.to_selector_name(error_msg_span)
                        })
                        .collect::<Vec<CompileResult<String>>>();
                    let mut buf = vec![];
                    for name in names {
                        match name.value {
                            Some(value) => buf.push(value),
                            None => return name,
                        }
                    }
                    buf
                };

                format!("e({})", variant_names.join(","))
            }
            _ => {
                return err(
                    vec![],
                    vec![CompileError::InvalidAbiType {
                        span: error_msg_span.clone(),
                    }],
                )
            }
        };
        ok(name, vec![], vec![])
    }
    /// Calculates the stack size of this type, to be used when allocating stack memory for it.
    /// This is _in words_!
    pub(crate) fn stack_size_of<'sc>(
        &self,
        err_span: &Span<'sc>,
    ) -> Result<u64, CompileError<'sc>> {
        match self {
            // Each char is a byte, so the size is the num of characters / 8
            // rounded up to the nearest word
            TypeInfo::Str(len) => Ok((len + 7) / 8),
            // Since things are unpacked, all unsigned integers are 64 bits.....for now
            TypeInfo::UnsignedInteger(_) | TypeInfo::Numeric => Ok(1),
            TypeInfo::Boolean => Ok(1),
            TypeInfo::Unit => Ok(0),
            TypeInfo::Byte => Ok(1),
            TypeInfo::B256 => Ok(4),
            TypeInfo::Enum { variant_types, .. } => {
                // the size of an enum is one word (for the tag) plus the maximum size
                // of any individual variant
                Ok(1 + variant_types
                    .iter()
                    .map(|x| -> Result<_, _> { look_up_type_id(x.r#type).stack_size_of(err_span) })
                    .collect::<Result<Vec<u64>, _>>()?
                    .into_iter()
                    .max()
                    .unwrap_or(0))
            }
            TypeInfo::Struct { fields, .. } => Ok(fields
                .iter()
                .map(|x| -> Result<_, _> {
                    resolve_type(x.r#type, err_span)
                        .expect("should be unreachable?")
                        .stack_size_of(err_span)
                })
                .collect::<Result<Vec<u64>, _>>()?
                .iter()
                .sum()),
            // `ContractCaller` types are unsized and used only in the type system for
            // calling methods
            TypeInfo::ContractCaller { .. } => Ok(0),
            TypeInfo::Contract => unreachable!("contract types are never instantiated"),
            TypeInfo::ErrorRecovery => unreachable!(),
            TypeInfo::Unknown | TypeInfo::Custom { .. } | TypeInfo::SelfType => {
                Err(CompileError::TypeMustBeKnown {
                    ty: self.friendly_type_str(),
                    span: err_span.clone(),
                })
            }
            TypeInfo::Ref(id) => look_up_type_id(*id).stack_size_of(err_span),
        }
    }
    pub(crate) fn is_copy_type(&self) -> bool {
        match self {
            TypeInfo::UnsignedInteger(_) | TypeInfo::Boolean | TypeInfo::Unit | TypeInfo::Byte => {
                true
            }
            _ => false,
        }
    }
}
