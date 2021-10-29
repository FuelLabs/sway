use crate::semantic_analysis::TypedExpression;
use crate::span::Span;
use crate::type_engine::{resolve_type, IntegerBits, TypeEngine, TypeInfo, TYPE_ENGINE};
use crate::{
    error::*,
    semantic_analysis::ast_node::{OwnedTypedStructField, TypedStructField},
    CallPath, Ident,
};
use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ResolvedType<'sc> {
    /// The number in a `Str` represents its size, which must be known at compile time
    Str(u64),
    UnsignedInteger(IntegerBits),
    Boolean,
    Unit,
    Byte,
    B256,
    Struct {
        name: Ident<'sc>,
        fields: Vec<TypedStructField<'sc>>,
    },
    Enum {
        name: Ident<'sc>,
        variant_types: Vec<ResolvedType<'sc>>,
    },
    /// Represents the contract's type as a whole. Used for implementing
    /// traits on the contract itself, to enforce a specific type of ABI.
    Contract,
    /// Represents a type which contains methods to issue a contract call.
    /// The specific contract is identified via the `Ident` within.
    ContractCaller {
        abi_name: CallPath<'sc>,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        address: Box<TypedExpression<'sc>>,
    },
    Function {
        from: Box<ResolvedType<'sc>>,
        to: Box<ResolvedType<'sc>>,
    },
    // used for recovering from errors in the ast
    ErrorRecovery,
}

impl ResolvedType<'_> {
    pub(crate) fn to_type_info(&self) -> TypeInfo {
        use ResolvedType::*;
        match self {
            Str(len) => TypeInfo::Str(*len),
            UnsignedInteger(bits) => TypeInfo::UnsignedInteger(*bits),
            Boolean => TypeInfo::Boolean,
            Unit => TypeInfo::Unit,
            Byte => TypeInfo::Byte,
            B256 => TypeInfo::B256,
            Struct { name, fields } => TypeInfo::Struct {
                name: name.primary_name.to_string(),
                fields: fields
                    .iter()
                    .map(TypedStructField::into_owned_typed_struct_field)
                    .collect::<Vec<OwnedTypedStructField>>(),
            },
            Enum {
                name,
                variant_types,
            } => todo!(),
            /// Represents the contract's type as a whole. Used for implementing
            /// traits on the contract itself, to enforce a specific type of ABI.
            Contract => TypeInfo::Contract,
            /// Represents a type which contains methods to issue a contract call.
            /// The specific contract is identified via the `Ident` within.
            ContractCaller { abi_name, address } => TypeInfo::ContractCaller {
                abi_name: abi_name.to_owned_call_path(),
                address: (*address).span.as_str().to_string(),
            },
            Function { from, to } => todo!("TypeInfo for functions"),
            // used for recovering from errors in the ast
            ErrorRecovery => TypeInfo::ErrorRecovery,
        }
    }
}

impl Default for ResolvedType<'_> {
    fn default() -> Self {
        ResolvedType::Unit
    }
}

impl<'sc> ResolvedType<'sc> {
    pub(crate) fn is_copy_type(&self) -> bool {
        match self {
            ResolvedType::UnsignedInteger(_)
            | ResolvedType::Boolean
            | ResolvedType::Unit
            | ResolvedType::Byte => true,
            _ => false,
        }
    }
    pub(crate) fn friendly_type_str(&self) -> String {
        use ResolvedType::*;
        match self {
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

            Unit => "()".into(),
            Byte => "byte".into(),
            B256 => "b256".into(),
            Struct {
                name: Ident { primary_name, .. },
                ..
            } => format!("struct {}", primary_name),
            Enum {
                name: Ident { primary_name, .. },
                ..
            } => format!("enum {}", primary_name),
            Contract => "contract".into(),
            ContractCaller { abi_name, .. } => {
                format!("{} contract caller", abi_name.suffix.primary_name)
            }
            Function { from, to } => format!(
                "fn({})->{}",
                from.friendly_type_str(),
                to.friendly_type_str()
            ),
            ErrorRecovery => "\"unknown due to error\"".into(),
        }
    }

    /// Calculates the stack size of this type, to be used when allocating stack memory for it.
    /// This is _in words_!
    pub(crate) fn stack_size_of(&self) -> u64 {
        let span = crate::Span {
            span: pest::Span::new("TODO(static span)", 0, 0).unwrap(),
            path: None,
        };

        match self {
            // Each char is a byte, so the size is the num of characters / 8
            // rounded up to the nearest word
            ResolvedType::Str(len) => (len + 7) / 8,
            // Since things are unpacked, all unsigned integers are 64 bits.....for now
            ResolvedType::UnsignedInteger(_) => 1,
            ResolvedType::Boolean => 1,
            ResolvedType::Unit => 0,
            ResolvedType::Byte => 1,
            ResolvedType::B256 => 4,
            ResolvedType::Enum { variant_types, .. } => {
                // the size of an enum is one word (for the tag) plus the maximum size
                // of any individual variant
                1 + variant_types
                    .iter()
                    .map(|x| x.stack_size_of())
                    .max()
                    .unwrap()
            }
            ResolvedType::Struct { fields, .. } => fields.iter().fold(0, |acc, x| {
                acc + (resolve_type(x.r#type, &x.span)
                    .expect("TODO(static spans)")
                    .stack_size_of(&span)
                    .expect("TODO(static spans)"))
            }),
            // `ContractCaller` types are unsized and used only in the type system for
            // calling methods
            ResolvedType::ContractCaller { .. } => 0,
            ResolvedType::Function { .. } => {
                unimplemented!("Function types have not yet been implemented.")
            }
            ResolvedType::Contract => unreachable!("contract types are never instantiated"),
            ResolvedType::ErrorRecovery => unreachable!(),
        }
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, ResolvedType::UnsignedInteger(_))
    }

    /// maps a type to a name that is used when constructing function selectors
    pub(crate) fn to_selector_name(
        &self,
        error_msg_span: &Span<'sc>,
    ) -> CompileResult<'sc, String> {
        use ResolvedType::*;
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
                        .map(|TypedStructField { r#type, .. }| {
                            CompileResult::from(resolve_type(*r#type, error_msg_span))
                                .flat_map(|x| x.to_selector_name(error_msg_span))
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
                        .map(|ty| ty.to_selector_name(error_msg_span))
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
}
