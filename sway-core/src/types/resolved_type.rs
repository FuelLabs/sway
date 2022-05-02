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
    /// used for recovering from errors in the ast
    ErrorRecovery,
}

impl Default for ResolvedType {
    fn default() -> Self {
        ResolvedType::Unit
    }
}

impl ResolvedType {
    pub(crate) fn is_copy_type(&self) -> bool {
        matches!(
            self,
            ResolvedType::Boolean
                | ResolvedType::Byte
                | ResolvedType::Unit
                | ResolvedType::UnsignedInteger(_)
        )
    }

    pub fn is_numeric(&self) -> bool {
        matches!(self, ResolvedType::UnsignedInteger(_))
    }
}
