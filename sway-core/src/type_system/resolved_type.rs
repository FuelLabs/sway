use crate::semantic_analysis::TypedExpression;
use crate::type_system::*;
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
    #[allow(dead_code)]
    Struct {
        name: Ident,
        fields: Vec<TypedStructField>,
    },
    #[allow(dead_code)]
    Enum {
        name: Ident,
        variant_types: Vec<ResolvedType>,
    },
    /// Represents the contract's type as a whole. Used for implementing
    /// traits on the contract itself, to enforce a specific type of ABI.
    #[allow(dead_code)]
    Contract,
    /// Represents a type which contains methods to issue a contract call.
    /// The specific contract is identified via the `Ident` within.
    #[allow(dead_code)]
    ContractCaller {
        abi_name: CallPath,
        #[derivative(PartialEq = "ignore", Hash = "ignore")]
        address: Box<TypedExpression>,
    },
    #[allow(dead_code)]
    Function {
        from: Box<ResolvedType>,
        to: Box<ResolvedType>,
    },
    /// used for recovering from errors in the ast
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn is_numeric(&self) -> bool {
        matches!(self, ResolvedType::UnsignedInteger(_))
    }
}
