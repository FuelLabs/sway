pub(crate) mod compile;
pub mod const_eval;
mod convert;
mod function;
mod lexical_map;
mod purity;
pub mod storage;
mod types;

use sway_error::error::CompileError;
use sway_ir::{Context, Kind};
use sway_types::span::Span;

pub(crate) use purity::{check_function_purity, PurityEnv};

use crate::{language::ty, Engines, ExperimentalFlags};

pub fn compile_program<'eng>(
    program: &ty::TyProgram,
    include_tests: bool,
    engines: &'eng Engines,
    experimental: ExperimentalFlags,
) -> Result<Context<'eng>, Vec<CompileError>> {
    let declaration_engine = engines.de();

    let test_fns = match include_tests {
        true => program.test_fns(declaration_engine).collect(),
        false => vec![],
    };

    let ty::TyProgram {
        kind,
        root,
        logged_types,
        messages_types,
        declarations,
        ..
    } = program;

    let logged_types = logged_types
        .iter()
        .map(|(log_id, type_id)| (*type_id, *log_id))
        .collect();

    let messages_types = messages_types
        .iter()
        .map(|(message_id, type_id)| (*type_id, *message_id))
        .collect();

    let mut ctx = Context::new(
        engines.se(),
        sway_ir::ExperimentalFlags {
            new_encoding: experimental.new_encoding,
        },
    );
    ctx.program_kind = match kind {
        ty::TyProgramKind::Script { .. } => Kind::Script,
        ty::TyProgramKind::Predicate { .. } => Kind::Predicate,
        ty::TyProgramKind::Contract { .. } => Kind::Contract,
        ty::TyProgramKind::Library { .. } => Kind::Library,
    };

    match kind {
        // predicates and scripts have the same codegen, their only difference is static
        // type-check time checks.
        ty::TyProgramKind::Script {
            entry_function: main_function,
        } => compile::compile_script(
            engines,
            &mut ctx,
            main_function,
            root.namespace.module(),
            declarations,
            &logged_types,
            &messages_types,
            &test_fns,
        ),
        ty::TyProgramKind::Predicate {
            entry_function: main_function,
        } => compile::compile_predicate(
            engines,
            &mut ctx,
            main_function,
            root.namespace.module(),
            declarations,
            &logged_types,
            &messages_types,
            &test_fns,
        ),
        ty::TyProgramKind::Contract {
            entry_function: main_function,
            abi_entries,
        } => compile::compile_contract(
            &mut ctx,
            main_function.as_ref(),
            abi_entries,
            root.namespace.module(),
            declarations,
            &logged_types,
            &messages_types,
            &test_fns,
            engines,
        ),
        ty::TyProgramKind::Library { .. } => compile::compile_library(
            engines,
            &mut ctx,
            root.namespace.module(),
            declarations,
            &logged_types,
            &messages_types,
            &test_fns,
        ),
    }?;

    //println!("{ctx}");

    ctx.verify().map_err(|ir_error: sway_ir::IrError| {
        vec![CompileError::InternalOwned(
            ir_error.to_string(),
            Span::dummy(),
        )]
    })
}
