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

pub fn compile_program(program: ty::TyProgram) -> Result<Context, CompileError> {
    let ty::TyProgram {
        kind,
        root,
        logged_types,
        ..
    } = program;

    let mut ctx = Context::default();
    match kind {
        ty::TyProgramKind::Script {
            main_function,
            declarations,
        } => compile::compile_script(
            &mut ctx,
            main_function,
            &root.namespace,
            declarations,
            &logged_types
                .into_iter()
                .map(|(log_id, type_id)| (type_id, log_id))
                .collect(),
        ),
        ty::TyProgramKind::Predicate {
            main_function,
            declarations,
        } => compile::compile_predicate(&mut ctx, main_function, &root.namespace, declarations),
        ty::TyProgramKind::Contract {
            abi_entries,
            declarations,
        } => compile::compile_contract(
            &mut ctx,
            abi_entries,
            &root.namespace,
            declarations,
            &logged_types
                .into_iter()
                .map(|(log_id, type_id)| (type_id, log_id))
                .collect(),
        ),
        ty::TyProgramKind::Library { .. } => unimplemented!("compile library to ir"),
    }?;
    ctx.verify()
        .map_err(|ir_error| CompileError::InternalOwned(ir_error.to_string(), Span::dummy()))
}
