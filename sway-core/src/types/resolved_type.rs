use crate::semantic_analysis::TypedExpression;
use crate::type_engine::*;
use crate::{semantic_analysis::ast_node::TypedStructField, CallPath, Ident};
use derivative::Derivative;

#[derive(Derivative)]
#[derivative(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ResolvedType {
    /// The number in a `Str` represents its size, which must be known at compile time
    Str(u64),
    UnsignedInteger(IntegerBits),
    Boolean,
    Unit,
    Byte,
    B256,
    Struct {
        name: Ident,
        fields: Vec<TypedStructField>,
    },
    Enum {
        name: Ident,
        variant_types: Vec<ResolvedType>,
    },
    /// Represents the contract's type as a whole. Used for implementing
    /// traits on the contract itself, to enforce a specific type of ABI.
    Contract,
    /// Represents a type which contains methods to issue a contract call.
    /// The specific contract is identified via the `Ident` within.
    ContractCaller {
        abi_name: CallPath,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        address: Box<TypedExpression>,
    },
    Function {
        from: Box<ResolvedType>,
        to: Box<ResolvedType>,
    },
    // used for recovering from errors in the ast
    ErrorRecovery,
}

impl Default for ResolvedType {
    fn default() -> Self {
        ResolvedType::Unit
    }
}

impl ResolvedType {
    /// Calculates the stack size of this type, to be used when allocating stack memory for it.
    /// This is _in words_!
    pub(crate) fn stack_size_of(&self) -> u64 {
        let span = sway_types::span::Span::new("TODO(static span)".into(), 0, 0, None).unwrap();

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
                    .size_in_words(&span)
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
}
