//! Various checks and heuristics that are naively run on sequences of opcodes.
//!
//! This is _not_ the place for optimization passes.
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Span;

use crate::asm_lang::{
    allocated_ops::{AllocatedOp, AllocatedOpcode},
    VirtualImmediate18,
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
pub(crate) fn check_script_opcodes(
    handler: &Handler,
    ops: &[AllocatedOp],
) -> Result<(), ErrorEmitted> {
    use AllocatedOpcode::*;
    let mut error_emitted = None;
    for op in ops {
        match op.opcode {
            GM(_, VirtualImmediate18 { value: 1..=2 }) => {
                error_emitted = Some(handler.emit_err(CompileError::GMFromExternalContext {
                    span: get_op_span(op),
                }));
            }
            MINT(..) => {
                error_emitted = Some(handler.emit_err(CompileError::MintFromExternalContext {
                    span: get_op_span(op),
                }));
            }
            BURN(..) => {
                error_emitted = Some(handler.emit_err(CompileError::BurnFromExternalContext {
                    span: get_op_span(op),
                }));
            }
            SWW(..) | SRW(..) | SRWQ(..) | SWWQ(..) => {
                error_emitted = Some(handler.emit_err(
                    CompileError::ContractStorageFromExternalContext {
                        span: get_op_span(op),
                    },
                ));
            }
            _ => (),
        }
    }

    if let Some(err) = error_emitted {
        // Abort compilation because the finalized asm contains opcodes invalid to a script.
        // Preemptively avoids the creation of scripts with opcodes not allowed at runtime.
        Err(err)
    } else {
        Ok(())
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
pub(crate) fn check_predicate_opcodes(
    handler: &Handler,
    ops: &[AllocatedOp],
) -> Result<(), ErrorEmitted> {
    use AllocatedOpcode::*;

    let mut error_emitted = None;

    for op in ops.iter() {
        let mut invalid_opcode = |name_str: &str| {
            error_emitted = Some(handler.emit_err(CompileError::InvalidOpcodeFromPredicate {
                opcode: name_str.to_string(),
                span: get_op_span(op),
            }));
        };
        match op.opcode.clone() {
            BAL(..) => invalid_opcode("BAL"),
            BHEI(..) => invalid_opcode("BHEI"),
            BHSH(..) => invalid_opcode("BHSH"),
            BURN(..) => invalid_opcode("BURN"),
            CALL(..) => invalid_opcode("CALL"),
            CB(..) => invalid_opcode("CB"),
            CCP(..) => invalid_opcode("CCP"),
            CROO(..) => invalid_opcode("CROO"),
            CSIZ(..) => invalid_opcode("CSIZ"),
            GM(_, VirtualImmediate18 { value: 1..=2 }) => {
                error_emitted = Some(handler.emit_err(CompileError::GMFromExternalContext {
                    span: get_op_span(op),
                }));
            }
            LDC(..) => invalid_opcode("LDC"),
            LOG(..) => invalid_opcode("LOG"),
            LOGD(..) => invalid_opcode("LOGD"),
            MINT(..) => invalid_opcode("MINT"),
            RETD(..) => invalid_opcode("RETD"),
            SMO(..) => invalid_opcode("SMO"),
            SRW(..) => invalid_opcode("SRW"),
            SRWQ(..) => invalid_opcode("SRWQ"),
            SWW(..) => invalid_opcode("SWW"),
            SWWQ(..) => invalid_opcode("SWWQ"),
            TIME(..) => invalid_opcode("TIME"),
            TR(..) => invalid_opcode("TR"),
            TRO(..) => invalid_opcode("TRO"),
            _ => (),
        };
    }

    if let Some(err) = error_emitted {
        // Abort compilation because the finalized asm contains opcodes invalid to a predicate.
        // Preemptively avoids the creation of predicates with opcodes not allowed at runtime.
        Err(err)
    } else {
        Ok(())
    }
}

fn get_op_span(op: &AllocatedOp) -> Span {
    let default_span =
        sway_types::span::Span::new("no span found for opcode".into(), 0, 1, None).unwrap();
    op.owning_span
        .clone()
        .unwrap_or_else(|| default_span.clone())
}
