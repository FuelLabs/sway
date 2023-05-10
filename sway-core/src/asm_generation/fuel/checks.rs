//! Various checks and heuristics that are naively run on sequences of opcodes.
//!
//! This is _not_ the place for optimization passes.
use sway_error::error::CompileError;
use sway_types::Span;

use crate::{
    asm_lang::{
        allocated_ops::{AllocatedOp, AllocatedOpcode},
        VirtualImmediate18,
    },
    error::*,
};

/// Checks if an opcode is one that cannot be executed from within a script.
/// If so, throw an error.
/// One example of disallowed code is as follows:
/// ```ignore
/// pub fn burn(amount: u64) {
///   asm(r1: amount) {
///     burn r1;
///   }
/// }
/// ```
pub(crate) fn check_script_opcodes(ops: &[AllocatedOp]) -> CompileResult<()> {
    use AllocatedOpcode::*;
    let mut errors = vec![];
    for op in ops {
        match op.opcode {
            GM(_, VirtualImmediate18 { value: 1..=2 }) => {
                errors.push(CompileError::GMFromExternalContext {
                    span: get_op_span(op),
                });
            }
            MINT(..) => {
                errors.push(CompileError::MintFromExternalContext {
                    span: get_op_span(op),
                });
            }
            BURN(..) => {
                errors.push(CompileError::BurnFromExternalContext {
                    span: get_op_span(op),
                });
            }
            SWW(..) | SRW(..) | SRWQ(..) | SWWQ(..) => {
                errors.push(CompileError::ContractStorageFromExternalContext {
                    span: get_op_span(op),
                });
            }
            _ => (),
        }
    }

    if errors.is_empty() {
        ok((), vec![], errors)
    } else {
        // Abort compilation because the finalized asm contains opcodes invalid to a script.
        // Preemptively avoids the creation of scripts with opcodes not allowed at runtime.
        err(vec![], errors)
    }
}

/// Checks if an opcode is one that cannot be executed from within a predicate.
/// If so, throw an error.
///
/// All contract opcodes are not allowed in predicates. Except for RVRT that can
/// be used to abort the predicate. One example of disallowed code is as follows:
/// ```ignore
/// pub fn burn(amount: u64) {
///   asm(r1: amount) {
///     burn r1;
///   }
/// }
/// ```
///
/// Jumping backwards is not allowed in predicates so JMP and JNE are not allowed and
/// the function verifies that the immediate of JI, JNEI, JNZI is greater than the opcode offset.
///
/// See: https://fuellabs.github.io/fuel-specs/master/vm/index.html?highlight=predicate#predicate-verification
pub(crate) fn check_predicate_opcodes(ops: &[AllocatedOp]) -> CompileResult<()> {
    use AllocatedOpcode::*;
    let mut errors = vec![];

    for op in ops.iter() {
        let invalid_opcode = |name_str: &str, errors: &mut Vec<CompileError>| {
            errors.push(CompileError::InvalidOpcodeFromPredicate {
                opcode: name_str.to_string(),
                span: get_op_span(op),
            });
        };
        match op.opcode.clone() {
            BAL(..) => invalid_opcode("BAL", &mut errors),
            BHEI(..) => invalid_opcode("BHEI", &mut errors),
            BHSH(..) => invalid_opcode("BHSH", &mut errors),
            BURN(..) => invalid_opcode("BURN", &mut errors),
            CALL(..) => invalid_opcode("CALL", &mut errors),
            CB(..) => invalid_opcode("CB", &mut errors),
            CCP(..) => invalid_opcode("CCP", &mut errors),
            CROO(..) => invalid_opcode("CROO", &mut errors),
            CSIZ(..) => invalid_opcode("CSIZ", &mut errors),
            GM(_, VirtualImmediate18 { value: 1..=2 }) => {
                errors.push(CompileError::GMFromExternalContext {
                    span: get_op_span(op),
                });
            }
            JMP(..) => invalid_opcode("JMP", &mut errors),
            JNE(..) => invalid_opcode("JNE", &mut errors),
            LDC(..) => invalid_opcode("LDC", &mut errors),
            LOG(..) => invalid_opcode("LOG", &mut errors),
            LOGD(..) => invalid_opcode("LOGD", &mut errors),
            MINT(..) => invalid_opcode("MINT", &mut errors),
            RETD(..) => invalid_opcode("RETD", &mut errors),
            SMO(..) => invalid_opcode("SMO", &mut errors),
            SRW(..) => invalid_opcode("SRW", &mut errors),
            SRWQ(..) => invalid_opcode("SRWQ", &mut errors),
            SWW(..) => invalid_opcode("SWW", &mut errors),
            SWWQ(..) => invalid_opcode("SWWQ", &mut errors),
            TIME(..) => invalid_opcode("TIME", &mut errors),
            TR(..) => invalid_opcode("TR", &mut errors),
            TRO(..) => invalid_opcode("TRO", &mut errors),
            _ => (),
        };
    }

    if errors.is_empty() {
        ok((), vec![], errors)
    } else {
        // Abort compilation because the finalized asm contains opcodes invalid to a predicate.
        // Preemptively avoids the creation of predicates with opcodes not allowed at runtime.
        err(vec![], errors)
    }
}

fn get_op_span(op: &AllocatedOp) -> Span {
    let default_span =
        sway_types::span::Span::new("no span found for opcode".into(), 0, 1, None).unwrap();
    op.owning_span
        .clone()
        .unwrap_or_else(|| default_span.clone())
}
