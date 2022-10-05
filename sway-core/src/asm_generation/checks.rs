//! Various checks and heuristics that are naively run on sequences of opcodes.
//!
//! This is _not_ the place for optimization passes.
use crate::asm_generation::FinalizedAsm;
use crate::error::*;
use crate::language::asm_lang::allocated_ops::{AllocatedOp, AllocatedOpcode};
use crate::language::asm_lang::*;

/// Checks for disallowed opcodes in non-contract code.
/// i.e., if this is a script or predicate, we can't use certain contract opcodes.
/// See https://github.com/FuelLabs/sway/issues/350 for details.
pub fn check_invalid_opcodes(asm: &FinalizedAsm) -> CompileResult<()> {
    match asm {
        FinalizedAsm::ContractAbi { .. } | FinalizedAsm::Library => ok((), vec![], vec![]),
        FinalizedAsm::ScriptMain {
            program_section, ..
        } => check_for_contract_opcodes(&program_section.ops[..]),
        FinalizedAsm::PredicateMain {
            program_section, ..
        } => check_for_contract_opcodes(&program_section.ops[..]),
    }
}

/// Checks if an opcode is one that can only be executed from within a contract. If so, throw an
/// error.
fn check_for_contract_opcodes(ops: &[AllocatedOp]) -> CompileResult<()> {
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
