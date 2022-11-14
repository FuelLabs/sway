//! Various checks and heuristics that are naively run on sequences of opcodes.
//!
//! This is _not_ the place for optimization passes.
use sway_error::error::CompileError;
use sway_types::Span;

use crate::asm_generation::{FinalizedAsm, ProgramKind};
use crate::asm_lang::allocated_ops::{AllocatedOp, AllocatedOpcode};
use crate::asm_lang::*;
use crate::error::*;

/// Checks for disallowed opcodes in non-contract code.
/// i.e., if this is a script or predicate, we can't use certain contract opcodes.
/// See https://github.com/FuelLabs/sway/issues/350 for details.
pub fn check_invalid_opcodes(asm: &FinalizedAsm) -> CompileResult<()> {
    match asm.program_kind {
        ProgramKind::Contract | ProgramKind::Library => ok((), vec![], vec![]),
        ProgramKind::Script => check_script_opcodes(&asm.program_section.ops[..]),
        ProgramKind::Predicate => check_predicate_opcodes(&asm.program_section.ops[..]),
    }
}

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
fn check_script_opcodes(ops: &[AllocatedOp]) -> CompileResult<()> {
    use AllocatedOpcode::*;
    let mut errors = vec![];
    for op in ops {
        match op.opcode {
            GM(_, VirtualImmediate18 { value: 1..=2 }) => {
                errors.push(CompileError::GMFromExternalContract {
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
fn check_predicate_opcodes(ops: &[AllocatedOp]) -> CompileResult<()> {
    use AllocatedOpcode::*;
    let mut errors = vec![];

    for (op, opcode_addr) in ops.iter().zip(0u32..) {
        macro_rules! invalid_opcode {
            ($name_str:literal) => {{
                errors.push(CompileError::InvalidOpcodeFromPredicate {
                    opcode: $name_str.to_string(),
                    span: get_op_span(op),
                });
            }};
        }
        macro_rules! invalid_backward_jump {
            ($name_str:literal) => {{
                errors.push(CompileError::InvalidBackwardJumpFromPredicate {
                    opcode: $name_str.to_string(),
                    span: get_op_span(op),
                });
            }};
        }
        match op.opcode.clone() {
            BAL(..) => invalid_opcode!("BAL"),
            BHEI(..) => invalid_opcode!("BHEI"),
            BHSH(..) => invalid_opcode!("BHSH"),
            BURN(..) => invalid_opcode!("BURN"),
            CALL(..) => invalid_opcode!("CALL"),
            CB(..) => invalid_opcode!("CB"),
            CCP(..) => invalid_opcode!("CCP"),
            CROO(..) => invalid_opcode!("CROO"),
            CSIZ(..) => invalid_opcode!("CSIZ"),
            GM(_, VirtualImmediate18 { value: 1..=2 }) => {
                errors.push(CompileError::GMFromExternalContract {
                    span: get_op_span(op),
                });
            }
            JI(imm) if imm.value <= opcode_addr => invalid_backward_jump!("JI"),
            JMP(..) => invalid_opcode!("JMP"),
            JNE(..) => invalid_opcode!("JNE"),
            JNEI(_, _, imm) if u32::from(imm.value) <= opcode_addr => {
                invalid_backward_jump!("JNEI")
            }
            JNZI(_, imm) if imm.value <= opcode_addr => invalid_backward_jump!("JNZI"),
            LDC(..) => invalid_opcode!("LDC"),
            LOG(..) => invalid_opcode!("LOG"),
            LOGD(..) => invalid_opcode!("LOGD"),
            MINT(..) => invalid_opcode!("MINT"),
            RETD(..) => invalid_opcode!("RETD"),
            SMO(..) => invalid_opcode!("SMO"),
            SRW(..) => invalid_opcode!("SRW"),
            SRWQ(..) => invalid_opcode!("SRWQ"),
            SWW(..) => invalid_opcode!("SWW"),
            SWWQ(..) => invalid_opcode!("SWWQ"),
            TIME(..) => invalid_opcode!("TIME"),
            TR(..) => invalid_opcode!("TR"),
            TRO(..) => invalid_opcode!("TRO"),
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
