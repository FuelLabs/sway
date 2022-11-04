//! Various checks and heuristics that are naively run on sequences of opcodes.
//!
//! This is _not_ the place for optimization passes.
use crate::asm_generation::{FinalizedAsm, ProgramKind};
use crate::asm_lang::allocated_ops::{AllocatedOp, AllocatedOpcode};
use crate::asm_lang::*;
use crate::error::*;

use sway_error::error::CompileError;

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
fn check_script_opcodes(ops: &[AllocatedOp]) -> CompileResult<()> {
    use AllocatedOpcode::*;
    let default_span =
        sway_types::span::Span::new("no span found for opcode".into(), 0, 1, None).unwrap();
    let mut errors = vec![];
    for op in ops {
        match op.opcode {
            GM(_, VirtualImmediate18 { value: 1 }) | GM(_, VirtualImmediate18 { value: 2 }) => {
                errors.push(CompileError::GMFromExternalContract {
                    span: op
                        .owning_span
                        .clone()
                        .unwrap_or_else(|| default_span.clone()),
                });
            }
            MINT(..) => {
                errors.push(CompileError::MintFromExternalContext {
                    span: op
                        .owning_span
                        .clone()
                        .unwrap_or_else(|| default_span.clone()),
                });
            }
            BURN(..) => {
                errors.push(CompileError::BurnFromExternalContext {
                    span: op
                        .owning_span
                        .clone()
                        .unwrap_or_else(|| default_span.clone()),
                });
            }
            SWW(..) | SRW(..) | SRWQ(..) | SWWQ(..) => {
                errors.push(CompileError::ContractStorageFromExternalContext {
                    span: op
                        .owning_span
                        .clone()
                        .unwrap_or_else(|| default_span.clone()),
                });
            }
            _ => (),
        }
    }

    if errors.is_empty() {
        ok((), vec![], errors)
    } else {
        err(vec![], errors)
    }
}

/// Checks if an opcode is one that cannot be executed from within a predicate.
/// If so, throw an error.
fn check_predicate_opcodes(ops: &[AllocatedOp]) -> CompileResult<()> {
    use AllocatedOpcode::*;
    let default_span =
        sway_types::span::Span::new("no span found for opcode".into(), 0, 1, None).unwrap();
    let mut errors = vec![];
    for op in ops {
        let invalid_opcode_opt = match op.opcode {
            BAL(..) => Some("BAL"),
            BHEI(..) => Some("BHEI"),
            BHSH(..) => Some("BHSN"),
            BURN(..) => Some("BURN"),
            CALL(..) => Some("CALL"),
            CB(..) => Some("CB"),
            CCP(..) => Some("CCP"),
            CROO(..) => Some("CROO"),
            CSIZ(..) => Some("CSIZ"),
            GM(_, VirtualImmediate18 { value: 1 }) | GM(_, VirtualImmediate18 { value: 2 }) => {
                Some("GM")
            }
            LDC(..) => Some("LDC"),
            LOG(..) => Some("LOG"),
            LOGD(..) => Some("LOGD"),
            MINT(..) => Some("MINT"),
            RETD(..) => Some("RETD"),
            //RVRT(..) => Some("RVRT"),
            SMO(..) => Some("SMO"),
            SRW(..) => Some("SRW"),
            SRWQ(..) => Some("SRWQ"),
            SWW(..) => Some("SWW"),
            SWWQ(..) => Some("SWWQ"),
            TIME(..) => Some("TIME"),
            TR(..) => Some("TR"),
            TRO(..) => Some("TRO"),
            _ => None,
        };
        if let Some(invalid_opcode) = invalid_opcode_opt {
            errors.push(CompileError::InvalidOpcodeFromPredicate {
                opcode: invalid_opcode.to_string(),
                span: op
                    .owning_span
                    .clone()
                    .unwrap_or_else(|| default_span.clone()),
            });
        }
    }

    if errors.is_empty() {
        ok((), vec![], errors)
    } else {
        err(vec![], errors)
    }
}
