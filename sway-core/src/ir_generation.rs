pub(crate) mod compile;
pub mod const_eval;
mod convert;
mod function;
mod lexical_map;
mod purity;
pub mod storage;
mod types;

use sway_error::error::CompileError;
use sway_ir::Context;
use sway_types::span::Span;

pub(crate) use purity::{check_function_purity, PurityEnv};

use crate::language::ty;

pub fn compile_program(
    program: ty::TyProgram,
    include_tests: bool,
) -> Result<Context, CompileError> {
    let test_fns = match include_tests {
        true => program.test_fns().collect(),
        false => vec![],
    };

    let ty::TyProgram {
        kind,
        root,
        logged_types,
        declarations,
        ..
    } = program;

    let logged_types = logged_types
        .into_iter()
        .map(|(log_id, type_id)| (type_id, log_id))
        .collect();

    let mut ctx = Context::default();
    match kind {
        // predicates and scripts have the same codegen, their only difference is static
        // type-check time checks.
        ty::TyProgramKind::Script { main_function } => compile::compile_script(
            &mut ctx,
            main_function,
            &root.namespace,
            declarations,
            &logged_types,
            test_fns,
        ),
        ty::TyProgramKind::Predicate { main_function } => compile::compile_predicate(
            &mut ctx,
            main_function,
            &root.namespace,
            declarations,
            &logged_types,
            test_fns,
        ),
        ty::TyProgramKind::Contract { abi_entries } => compile::compile_contract(
            &mut ctx,
            abi_entries,
            &root.namespace,
            declarations,
            &logged_types,
            test_fns,
        ),
        ty::TyProgramKind::Library { .. } => compile::compile_library(
            &mut ctx,
            &root.namespace,
            declarations,
            &logged_types,
            test_fns,
        ),
    }?;
    ctx.verify()
        .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))
}
