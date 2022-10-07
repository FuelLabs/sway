pub(crate) mod compile;
pub mod const_eval;
mod convert;
mod function;
mod lexical_map;
mod purity;
pub mod storage;
mod types;

use crate::{
    error::CompileError,
    language::ty::{TyProgram, TyProgramKind},
};

use sway_ir::Context;
use sway_types::span::Span;

pub(crate) use purity::PurityChecker;

pub fn compile_program(program: TyProgram) -> Result<Context, CompileError> {
    let TyProgram { kind, root, .. } = program;

    let mut ctx = Context::default();
    match kind {
        TyProgramKind::Script {
            main_function,
            declarations,
        }
        | TyProgramKind::Predicate {
            main_function,
            declarations,
            // predicates and scripts have the same codegen, their only difference is static
            // type-check time checks.
        } => compile::compile_script(&mut ctx, main_function, &root.namespace, declarations),
        TyProgramKind::Contract {
            abi_entries,
            declarations,
        } => compile::compile_contract(&mut ctx, abi_entries, &root.namespace, declarations),
        TyProgramKind::Library { .. } => unimplemented!("compile library to ir"),
    }?;
    ctx.verify()
        .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))
}
