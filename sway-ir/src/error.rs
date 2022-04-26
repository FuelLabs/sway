/// These errors are for internal IR failures, not designed to be useful to a Sway developer, but
/// more for users of the `sway-ir` crate, i.e., compiler developers.
///
/// XXX They're not very rich and could do with a little more verbosity.

#[derive(Debug)]
pub enum IrError {
    FunctionLocalClobbered(String, String),
    InvalidMetadatum,
    MisplacedTerminator(String),
    MissingBlock(String),
    MissingTerminator(String),
    ParseFailure(String, String),
    ValueNotFound(String),

    VerifyAccessElementInconsistentTypes,
    VerifyAccessElementOnNonArray,
    VerifyAccessElementNonIntIndex,
    VerifyAccessValueInconsistentTypes,
    VerifyAccessValueInvalidIndices,
    VerifyAccessValueOnNonStruct,
    VerifyArgumentValueIsNotArgument(String),
    VerifyBitcastUnknownSourceType,
    VerifyBitcastFromNonCopyType(String),
    VerifyBitcastToNonCopyType(String),
    VerifyBitcastBetweenInvalidTypes(String, String),
    VerifyBranchToMissingBlock(String),
    VerifyCallArgTypeMismatch(String),
    VerifyCallToMissingFunction(String),
    VerifyCmpBadTypes(String, String),
    VerifyCmpTypeMismatch(String, String),
    VerifyCmpUnknownTypes,
    VerifyConditionExprNotABool,
    VerifyContractCallBadTypes(String),
    VerifyGetNonExistentPointer,
    VerifyInsertElementOfIncorrectType,
    VerifyInsertValueOfIncorrectType,
    VerifyLoadFromNonPointer,
    VerifyLoadNonExistentPointer,
    VerifyMismatchedReturnTypes(String),
    VerifyPhiFromMissingBlock(String),
    VerifyPhiInconsistentTypes,
    VerifyPhiNonUniqueLabels,
    VerifyPtrCastFromNonPointer,
    VerifyStateKeyBadType,
    VerifyStateDestBadType(String),
    VerifyStoreMismatchedTypes,
    VerifyStoreNonExistentPointer,
    VerifyStoreToNonPointer,
    VerifyUntypedValuePassedToFunction,
}

use std::fmt;

impl fmt::Display for IrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            IrError::FunctionLocalClobbered(fn_str, var_str) => write!(
                f,
                "Local storage for function {fn_str} already has an entry for variable {var_str}"
            ),
            IrError::InvalidMetadatum => write!(f, "Unable to convert from invalid metadatum."),
            IrError::MisplacedTerminator(blk_str) => {
                write!(f, "Block {blk_str} has a misplaced terminator.")
            }
            IrError::MissingBlock(blk_str) => write!(f, "Unable to find block {blk_str}."),
            IrError::MissingTerminator(blk_str) => {
                write!(f, "Block {blk_str} is missing its terminator.")
            }
            IrError::ParseFailure(expecting, found) => {
                write!(f, "Parse failure: expecting '{expecting}', found '{found}'")
            }
            IrError::ValueNotFound(reason) => {
                write!(f, "Invalid value: {reason}")
            }

            // Verification failures:
            IrError::VerifyAccessElementNonIntIndex => {
                write!(
                    f,
                    "Verification failed: Array index is not an integral type."
                )
            }
            IrError::VerifyAccessElementInconsistentTypes => {
                write!(
                    f,
                    "Verification failed: Array type mismatch in when accessing array element."
                )
            }
            IrError::VerifyAccessElementOnNonArray => {
                write!(
                    f,
                    "Verification failed: Attempt to access an element from a non array."
                )
            }
            IrError::VerifyAccessValueInconsistentTypes => {
                write!(f, "Verification failed: Struct field type mismatch.")
            }
            IrError::VerifyAccessValueInvalidIndices => {
                write!(
                    f,
                    "Verification failed: Struct index to non-existent field."
                )
            }
            IrError::VerifyAccessValueOnNonStruct => {
                write!(
                    f,
                    "Verification failed: Attempt to access a field from a non struct."
                )
            }
            IrError::VerifyArgumentValueIsNotArgument(callee) => write!(
                f,
                "Verification failed: Argument specifier for function '{callee}' is not an argument value."
            ),
            IrError::VerifyBitcastUnknownSourceType => write!(
                f,
                "Verification failed: Bitcast unable to determine source type."
            ),
            IrError::VerifyBitcastFromNonCopyType(ty) => write!(
                f,
                "Verification failed: Bitcast cannot be from a {ty}."
            ),
            IrError::VerifyBitcastToNonCopyType(ty) => write!(
                f,
                "Verification failed: Bitcast cannot be to a {ty}."
            ),
            IrError::VerifyBitcastBetweenInvalidTypes(from_ty, to_ty) => write!(
                f,
                "Verification failed: Bitcast not allowed from a {from_ty} to a {to_ty}."
            ),
            IrError::VerifyBranchToMissingBlock(label) => {
                write!(
                    f,
                    "Verification failed: Branch to block '{label}' is not a block in the current function."
                )
            }
            IrError::VerifyCallArgTypeMismatch(callee) => {
                write!(
                    f,
                    "Verification failed: Type mismatch found for call to '{callee}'."
                )
            }
            IrError::VerifyCallToMissingFunction(callee) => {
                write!(
                    f,
                    "Verification failed: Call to invalid function '{callee}'."
                )
            }
            IrError::VerifyCmpBadTypes(lhs_ty, rhs_ty) => {
                write!(
                    f,
                    "Verification failed: Cannot compare non-integer types {lhs_ty} and {rhs_ty}."
                )
            }
            IrError::VerifyCmpTypeMismatch(lhs_ty, rhs_ty) => {
                write!(
                    f,
                    "Verification failed: Cannot compare values with different widths of {lhs_ty} and {rhs_ty}."
                )
            }
            IrError::VerifyCmpUnknownTypes => {
                write!(
                    f,
                    "Verification failed: Unable to determine type(s) of compared value(s)."
                )
            }
            IrError::VerifyConditionExprNotABool => {
                write!(
                    f,
                    "Verification failed: Expression used for conditional is not a boolean."
                )
            }
            IrError::VerifyContractCallBadTypes(arg_name) => {
                write!(
                    f,
                    "Verification failed: Argument {arg_name} passed to contract call has the incorrect type."
                )
            }
            IrError::VerifyGetNonExistentPointer => {
                write!(
                    f,
                    "Verification failed: Attempt to get pointer not found in function locals."
                )
            }
            IrError::VerifyInsertElementOfIncorrectType => {
                write!(
                    f,
                    "Verification failed: Attempt to insert value of incorrect type into an array."
                )
            }
            IrError::VerifyInsertValueOfIncorrectType => {
                write!(
                    f,
                    "Verification failed: Attempt to insert value of incorrect type into a struct."
                )
            }
            IrError::VerifyLoadFromNonPointer => {
                write!(f, "Verification failed: Load must be from a pointer.")
            }
            IrError::VerifyLoadNonExistentPointer => write!(
                f,
                "Verification failed: Attempt to load from a pointer not found in function locals."
            ),
            IrError::VerifyMismatchedReturnTypes(fn_str) => write!(
                f,
                "Verification failed: Function {fn_str} return type must match its RET instructions."
            ),
            IrError::VerifyPhiFromMissingBlock(label) => {
                write!(
                    f,
                    "Verification failed: PHI has a block '{label}'not from the current function."
                )
            }
            IrError::VerifyPhiInconsistentTypes => {
                write!(f, "Verification failed: PHI has inconsistent types.")
            }
            IrError::VerifyPhiNonUniqueLabels => {
                write!(f, "Verification failed: PHI must have unique block labels.")
            }
            IrError::VerifyPtrCastFromNonPointer => {
                write!(
                    f,
                    "Verification failed: Pointer cast from non pointer value."
                )
            }
            IrError::VerifyStateKeyBadType => {
                write!(
                    f,
                    "Verification failed: State load or store key must be a b256 pointer."
                )
            }
            IrError::VerifyStateDestBadType(ty) => {
                write!(
                    f,
                    "Verification failed: State access operation must be to a {ty} pointer."
                )
            }
            IrError::VerifyStoreMismatchedTypes => {
                write!(
                    f,
                    "Verification failed: Store value and pointer type mismatch."
                )
            }
            IrError::VerifyStoreNonExistentPointer => write!(
                f,
                "Verification failed: Attempt to store to a pointer not found in function locals."
            ),
            IrError::VerifyStoreToNonPointer => write!(f, "Store must be to a pointer."),
            IrError::VerifyUntypedValuePassedToFunction => write!(
                f,
                "Verification failed: An untyped/void value has been passed to a function call."
            ),
        }
    }
}
