/// These errors are for internal IR failures, not designed to be useful to a Sway developer, but
/// more for users of the `sway-ir` crate, i.e., compiler developers.
///
/// XXX They're not very rich and could do with a little more verbosity.

#[derive(Debug)]
pub enum IrError {
    FunctionLocalClobbered(String, String),
    InvalidMetadatum(String),
    InvalidPhi,
    MisplacedTerminator(String),
    MissingBlock(String),
    MissingTerminator(String),
    ParseFailure(String, String),
    RemoveMissingBlock(String),
    ValueNotFound(String),
    InconsistentParent(String, String, String),

    VerifyArgumentValueIsNotArgument(String),
    VerifyUnaryOpIncorrectArgType,
    VerifyBinaryOpIncorrectArgType,
    VerifyBitcastBetweenInvalidTypes(String, String),
    VerifyBitcastUnknownSourceType,
    VerifyEntryBlockHasPredecessors(String, Vec<String>),
    VerifyBlockArgMalformed,
    VerifyBranchParamsMismatch,
    VerifyBranchToMissingBlock(String),
    VerifyCallArgTypeMismatch(String, String, String),
    VerifyCallToMissingFunction(String),
    VerifyCmpBadTypes(String, String),
    VerifyCmpTypeMismatch(String, String),
    VerifyCmpUnknownTypes,
    VerifyConditionExprNotABool,
    VerifyContractCallBadTypes(String),
    VerifyGepElementTypeNonPointer,
    VerifyGepFromNonPointer(String, Option<Value>),
    VerifyGepInconsistentTypes(String, Option<crate::Value>),
    VerifyGepOnNonAggregate,
    VerifyGetNonExistentLocalVarPointer,
    VerifyGetNonExistentGlobalVarPointer,
    VerifyGetNonExistentConfigPointer,
    VerifyGetNonExistentStorageKeyPointer,
    VerifyGlobalMissingInitializer(String),
    VerifyInsertElementOfIncorrectType,
    VerifyInsertValueOfIncorrectType,
    VerifyIntToPtrFromNonIntegerType(String),
    VerifyIntToPtrToNonPointer(String),
    VerifyIntToPtrUnknownSourceType,
    VerifyInvalidGtfIndexType,
    VerifyLoadFromNonPointer(String),
    VerifyAllocToNonPointer(String),
    VerifyLocalMissingInitializer(String, String),
    VerifyLogId,
    VerifyLogMismatchedTypes,
    VerifyLogEventDataVersion(u8),
    VerifyLogEventDataInvalid(String),
    VerifyMemcopyNonPointer(String),
    VerifyMemcopyMismatchedTypes(String, String),
    VerifyMemClearValNonPointer(String),
    VerifyPtrCastFromNonPointer(String),
    VerifyPtrCastToNonPointer(String),
    VerifyPtrToIntToNonInteger(String),
    VerifyReturnMismatchedTypes(String),
    VerifyRevertCodeBadType,
    VerifySmoBadMessageType,
    VerifySmoCoins,
    VerifySmoMessageSize,
    VerifySmoRecipientNonPointer(String),
    VerifySmoMessageNonPointer(String),
    VerifySmoRecipientBadType,
    VerifyStateAccessNumOfSlots,
    VerifyStateAccessQuadNonPointer(String),
    VerifyStateDestBadType(String),
    VerifyStateKeyBadType,
    VerifyStateKeyNonPointer(String),
    VerifyStoreMismatchedTypes(Option<Value>),
    VerifyStoreToNonPointer(String),
    VerifyUntypedValuePassedToFunction,
}
impl IrError {
    pub(crate) fn get_problematic_value(&self) -> Option<&Value> {
        match self {
            Self::VerifyGepFromNonPointer(_, v) => v.as_ref(),
            Self::VerifyGepInconsistentTypes(_, v) => v.as_ref(),
            Self::VerifyStoreMismatchedTypes(v) => v.as_ref(),
            _ => None,
        }
    }
}

impl std::error::Error for IrError {}

use std::fmt;

use crate::Value;
use itertools::Itertools;

impl fmt::Display for IrError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            IrError::FunctionLocalClobbered(fn_str, var_str) => write!(
                f,
                "Local storage for function {fn_str} already has an entry for variable {var_str}."
            ),
            IrError::InvalidMetadatum(why_str) => {
                write!(f, "Unable to convert from invalid metadatum: {why_str}.")
            }
            IrError::InvalidPhi => write!(
                f,
                "Phi instruction has invalid block or value reference list."
            ),
            IrError::MisplacedTerminator(blk_str) => {
                write!(f, "Block {blk_str} has a misplaced terminator.")
            }
            IrError::MissingBlock(blk_str) => write!(f, "Unable to find block {blk_str}."),
            IrError::MissingTerminator(blk_str) => {
                write!(f, "Block {blk_str} is missing its terminator.")
            }
            IrError::ParseFailure(expecting, found) => {
                write!(
                    f,
                    "Parse failure: expecting '{expecting}', found '{found}'."
                )
            }
            IrError::RemoveMissingBlock(blk_str) => {
                write!(f, "Unable to remove block {blk_str}; not found.")
            }
            IrError::ValueNotFound(reason) => {
                write!(f, "Invalid value: {reason}.")
            }
            IrError::InconsistentParent(entity, expected_parent, found_parent) => {
                write!(
                                    f,
                                    "For IR Entity (module/function/block) {entity}, expected parent to be {expected_parent}, \
                    but found {found_parent}."
                                )
            }
            IrError::VerifyArgumentValueIsNotArgument(callee) => write!(
                f,
                "Verification failed: Argument specifier for function '{callee}' is not an \
                argument value."
            ),
            IrError::VerifyBitcastUnknownSourceType => write!(
                f,
                "Verification failed: Bitcast unable to determine source type."
            ),
            IrError::VerifyBitcastBetweenInvalidTypes(from_ty, to_ty) => write!(
                f,
                "Verification failed: Bitcast not allowed from a {from_ty} to a {to_ty}."
            ),
            IrError::VerifyUnaryOpIncorrectArgType => {
                write!(
                    f,
                    "Verification failed: Incorrect argument type for unary op"
                )
            }
            IrError::VerifyBinaryOpIncorrectArgType => {
                write!(
                    f,
                    "Verification failed: Incorrect argument type(s) for binary op"
                )
            }
            IrError::VerifyBranchToMissingBlock(label) => {
                write!(
                    f,
                    "Verification failed: \
                    Branch to block '{label}' is not a block in the current function."
                )
            }
            IrError::VerifyCallArgTypeMismatch(callee, caller_ty, callee_ty) => {
                write!(
                                    f,
                                    "Verification failed: Type mismatch found for call to '{callee}': {caller_ty} is not a {callee_ty}."
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
                    "Verification failed: \
                    Cannot compare values with different widths of {lhs_ty} and {rhs_ty}."
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
                    "Verification failed: \
                    Argument {arg_name} passed to contract call has the incorrect type."
                )
            }
            IrError::VerifyGepElementTypeNonPointer => {
                write!(f, "Verification failed: GEP on a non-pointer.")
            }
            IrError::VerifyGepInconsistentTypes(error, _) => {
                write!(
                    f,
                    "Verification failed: Struct field type mismatch: ({error})."
                )
            }
            IrError::VerifyGepFromNonPointer(ty, _) => {
                write!(
                    f,
                    "Verification failed: Struct access must be to a pointer value, not a {ty}."
                )
            }
            IrError::VerifyGepOnNonAggregate => {
                write!(
                    f,
                    "Verification failed: Attempt to access a field from a non struct."
                )
            }
            IrError::VerifyGetNonExistentLocalVarPointer => {
                write!(
                    f,
                    "Verification failed: Attempt to get pointer not found in function local variables."
                )
            }
            IrError::VerifyGetNonExistentGlobalVarPointer => {
                write!(
                    f,
                    "Verification failed: Attempt to get pointer not found in module global variables."
                )
            }
            IrError::VerifyGetNonExistentConfigPointer => {
                write!(
                    f,
                    "Verification failed: Attempt to get pointer not found in module configurables."
                )
            }
            IrError::VerifyGetNonExistentStorageKeyPointer => {
                write!(
                    f,
                    "Verification failed: Attempt to get pointer not found in module storage keys."
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
            IrError::VerifyIntToPtrFromNonIntegerType(ty) => {
                write!(f, "Verification failed: int_to_ptr cannot be from a {ty}.")
            }
            IrError::VerifyIntToPtrToNonPointer(ty) => {
                write!(
                    f,
                    "Verification failed: int_to_ptr cannot be to a non-pointer {ty}."
                )
            }
            IrError::VerifyIntToPtrUnknownSourceType => write!(
                f,
                "Verification failed: int_to_ptr unable to determine source type."
            ),
            IrError::VerifyLoadFromNonPointer(ty) => {
                write!(
                    f,
                    "Verification failed: Load cannot be from a non-pointer {ty}."
                )
            }
            IrError::VerifyAllocToNonPointer(ty) => {
                write!(
                    f,
                    "Verification failed: Alloc must store a pointer to the type being allocated, \
                     but it contains {ty}."
                )
            }
            IrError::VerifyMemcopyNonPointer(ty) => {
                write!(
                    f,
                    "Verification failed: mem_copy cannot be to or from a non-pointer {ty}.",
                )
            }
            IrError::VerifyMemcopyMismatchedTypes(dst_ty, src_ty) => {
                write!(
                    f,
                    "Verification failed: mem_copy cannot be from {src_ty} pointer to {dst_ty} \
                    pointer.",
                )
            }
            IrError::VerifyMemClearValNonPointer(ty) => {
                write!(
                    f,
                    "Verification failed: mem_clear_val argument is not a pointer {ty}.",
                )
            }
            IrError::VerifyReturnMismatchedTypes(fn_str) => write!(
                f,
                "Verification failed: \
                Function {fn_str} return type must match its RET instructions."
            ),
            IrError::VerifyEntryBlockHasPredecessors(function_name, predecessors) => {
                let plural_s = if predecessors.len() == 1 { "" } else { "s" };
                write!(
                                    f,
                                    "Verification failed: Entry block of the function \"{function_name}\" has {}predecessor{}. \
                     The predecessor{} {} {}.",
                                    if predecessors.len() == 1 {
                                        "a "
                                    } else {
                                        ""
                                    },
                                    plural_s,
                                    plural_s,
                                    if predecessors.len() == 1 {
                                        "is"
                                    } else {
                                        "are"
                                    },
                                    predecessors.iter().map(|block_label| format!("\"{block_label}\"")).collect_vec().join(", ")
                                )
            }
            IrError::VerifyBlockArgMalformed => {
                write!(f, "Verification failed: Block argument is malformed")
            }
            IrError::VerifyBranchParamsMismatch => {
                write!(
                    f,
                    "Verification failed: Block parameter passed in branch is malformed"
                )
            }
            IrError::VerifyPtrCastFromNonPointer(ty) => {
                write!(
                    f,
                    "Verification failed: Pointer cast from non pointer {ty}."
                )
            }
            IrError::VerifyPtrCastToNonPointer(ty) => {
                write!(f, "Verification failed: Pointer cast to non pointer {ty}.")
            }
            IrError::VerifyPtrToIntToNonInteger(ty) => {
                write!(f, "Verification failed: Pointer cast to non integer {ty}.")
            }
            IrError::VerifyStateAccessNumOfSlots => {
                write!(
                    f,
                    "Verification failed: Number of slots for state access must be an integer."
                )
            }
            IrError::VerifyStateAccessQuadNonPointer(ty) => {
                write!(
                    f,
                    "Verification failed: \
                    State quad access must be to or from a pointer, not a {ty}."
                )
            }
            IrError::VerifyStateKeyBadType => {
                write!(
                    f,
                    "Verification failed: State load or store key must be a b256 pointer."
                )
            }
            IrError::VerifyStateKeyNonPointer(ty) => {
                write!(
                    f,
                    "Verification failed: State load or store key must be a pointer, not a {ty}."
                )
            }
            IrError::VerifyStateDestBadType(ty) => {
                write!(
                    f,
                    "Verification failed: State access operation must be to a {ty} pointer."
                )
            }
            IrError::VerifyStoreMismatchedTypes(_) => {
                write!(
                    f,
                    "Verification failed: Store value and pointer type mismatch."
                )
            }
            IrError::VerifyStoreToNonPointer(ty) => {
                write!(f, "Store must be to a pointer, not a {ty}.")
            }
            IrError::VerifyUntypedValuePassedToFunction => write!(
                f,
                "Verification failed: An untyped/void value has been passed to a function call."
            ),
            IrError::VerifyInvalidGtfIndexType => write!(
                f,
                "Verification failed: An non-integer value has been passed to a 'gtf' instruction."
            ),
            IrError::VerifyLogId => {
                write!(f, "Verification failed: log ID must be an integer.")
            }
            IrError::VerifyLogMismatchedTypes => {
                write!(
                    f,
                    "Verification failed: log type must match the type of the value being logged."
                )
            }
            IrError::VerifyLogEventDataVersion(version) => {
                write!(
                    f,
                    "Verification failed: unsupported log event metadata version {version}."
                )
            }
            IrError::VerifyLogEventDataInvalid(reason) => {
                write!(
                    f,
                    "Verification failed: invalid log event metadata ({reason})."
                )
            }
            IrError::VerifyRevertCodeBadType => {
                write!(
                    f,
                    "Verification failed: error code for revert must be a u64."
                )
            }
            IrError::VerifySmoRecipientBadType => {
                write!(
                    f,
                    "Verification failed: the `smo` must have a `b256` as its first argument."
                )
            }
            IrError::VerifySmoBadMessageType => {
                write!(
                    f,
                    "Verification failed: the second arg of of `smo` must be a struct."
                )
            }
            IrError::VerifySmoMessageSize => {
                write!(
                    f,
                    "Verification failed: smo message size must be an integer."
                )
            }
            IrError::VerifySmoRecipientNonPointer(ty) => {
                write!(
                    f,
                    "Verification failed: the first arg of `smo` cannot be a non-pointer of {ty}."
                )
            }
            IrError::VerifySmoMessageNonPointer(ty) => {
                write!(
                    f,
                    "Verification failed: the second arg of `smo` cannot be a non-pointer of {ty}."
                )
            }
            IrError::VerifySmoCoins => {
                write!(
                    f,
                    "Verification failed: smo coins value must be an integer."
                )
            }
            IrError::VerifyGlobalMissingInitializer(global_name) => {
                write!(
                    f,
                    "Verification failed: Immutable global variable {global_name}\
                    is missing an initializer."
                )
            }
            IrError::VerifyLocalMissingInitializer(local_name, func_name) => {
                write!(
                    f,
                    "Verification failed: Immutable local variable {local_name} in function \
                    {func_name} is missing an initializer."
                )
            }
        }
    }
}
