pub(crate) mod compile;
pub mod const_eval;
mod convert;
mod function;
mod lexical_map;
mod purity;
pub mod storage;
mod types;

use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hasher},
};

use sway_error::error::CompileError;
use sway_features::ExperimentalFeatures;
use sway_ir::{Context, Function, Kind, Module};
use sway_types::{span::Span, Ident};

pub(crate) use purity::{check_function_purity, PurityEnv};

use crate::{
    engine_threading::HashWithEngines,
    language::ty,
    metadata::MetadataManager,
    types::{LogId, MessageId},
    Engines, PanicOccurrences, TypeId,
};

type FnKey = u64;

/// Every compiled function needs to go through this cache for two reasons:
/// 1 - to have its IR name unique;
/// 2 - to avoid being compiled twice.
#[derive(Default)]
pub(crate) struct CompiledFunctionCache {
    recreated_fns: HashMap<FnKey, Function>,
}

impl CompiledFunctionCache {
    #[allow(clippy::too_many_arguments)]
    fn ty_function_decl_to_unique_function(
        &mut self,
        engines: &Engines,
        context: &mut Context,
        module: Module,
        md_mgr: &mut MetadataManager,
        decl: &ty::TyFunctionDecl,
        logged_types_map: &HashMap<TypeId, LogId>,
        messages_types_map: &HashMap<TypeId, MessageId>,
        panic_occurrences: &mut PanicOccurrences,
    ) -> Result<Function, CompileError> {
        // The compiler inlines everything very lazily.  Function calls include the body of the
        // callee (i.e., the callee_body arg above). Library functions are provided in an initial
        // namespace from Forc and when the parser builds the AST (or is it during type checking?)
        // these function bodies are embedded.
        //
        // Here we build little single-use instantiations of the callee and then call them.  Naming
        // is not yet absolute so we must ensure the function names are unique.
        //
        // Eventually we need to Do It Properly and inline into the AST only when necessary, and
        // compile the standard library to an actual module.
        //
        // Get the callee from the cache if we've already compiled it.  We can't insert it with
        // .entry() since `compile_function()` returns a Result we need to handle.  The key to our
        // cache, to uniquely identify a function instance, is the span and the type IDs of any
        // args and type parameters.  It's using the Sway types rather than IR types, which would
        // be more accurate but also more fiddly.

        let mut hasher = DefaultHasher::default();
        decl.hash(&mut hasher, engines);
        let fn_key = hasher.finish();

        let (fn_key, item) = (Some(fn_key), self.recreated_fns.get(&fn_key).copied());
        let new_callee = match None {
            Some(func) => func,
            None => {
                let name = Ident::new(Span::from_string(format!(
                    "{}_{}",
                    decl.name,
                    context.get_unique_symbol_id()
                )));
                let callee_fn_decl = ty::TyFunctionDecl {
                    type_parameters: Vec::new(),
                    name,
                    parameters: decl.parameters.clone(),
                    ..decl.clone()
                };
                // Entry functions are already compiled at the top level
                // when compiling scripts, predicates, contracts, and libraries.
                let is_entry = false;
                let is_original_entry = callee_fn_decl.is_main() || callee_fn_decl.is_test();
                let new_func = compile::compile_function(
                    engines,
                    context,
                    md_mgr,
                    module,
                    &callee_fn_decl,
                    &decl.name,
                    logged_types_map,
                    messages_types_map,
                    panic_occurrences,
                    is_entry,
                    is_original_entry,
                    None,
                    self,
                )
                .map_err(|mut x| x.pop().unwrap())?
                .unwrap();

                if let Some(fn_key) = fn_key {
                    self.recreated_fns.insert(fn_key, new_func);
                }

                new_func
            }
        };

        Ok(new_callee)
    }
}

pub fn compile_program<'a>(
    program: &ty::TyProgram,
    panic_occurrences: &'a mut PanicOccurrences,
    include_tests: bool,
    engines: &'a Engines,
    experimental: ExperimentalFeatures,
) -> Result<Context<'a>, Vec<CompileError>> {
    let declaration_engine = engines.de();

    let test_fns = match include_tests {
        true => program.test_fns(declaration_engine).collect(),
        false => vec![],
    };

    let ty::TyProgram {
        kind,
        namespace,
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

    let mut ctx = Context::new(engines.se(), experimental);
    ctx.program_kind = match kind {
        ty::TyProgramKind::Script { .. } => Kind::Script,
        ty::TyProgramKind::Predicate { .. } => Kind::Predicate,
        ty::TyProgramKind::Contract { .. } => Kind::Contract,
        ty::TyProgramKind::Library { .. } => Kind::Library,
    };

    let mut cache = CompiledFunctionCache::default();

    match kind {
        // Predicates and scripts have the same codegen, their only difference is static
        // type-check time checks.
        ty::TyProgramKind::Script { entry_function, .. } => compile::compile_script(
            engines,
            &mut ctx,
            entry_function,
            namespace,
            &logged_types,
            &messages_types,
            panic_occurrences,
            &test_fns,
            &mut cache,
        ),
        ty::TyProgramKind::Predicate { entry_function, .. } => compile::compile_predicate(
            engines,
            &mut ctx,
            entry_function,
            namespace,
            &logged_types,
            &messages_types,
            panic_occurrences,
            &test_fns,
            &mut cache,
        ),
        ty::TyProgramKind::Contract {
            entry_function,
            abi_entries,
        } => compile::compile_contract(
            &mut ctx,
            entry_function.as_ref(),
            abi_entries,
            namespace,
            declarations,
            &logged_types,
            &messages_types,
            panic_occurrences,
            &test_fns,
            engines,
            &mut cache,
        ),
        ty::TyProgramKind::Library { .. } => compile::compile_library(
            engines,
            &mut ctx,
            namespace,
            &logged_types,
            &messages_types,
            panic_occurrences,
            &test_fns,
            &mut cache,
        ),
    }?;

    ctx.verify().map_err(|ir_error: sway_ir::IrError| {
        vec![CompileError::InternalOwned(
            ir_error.to_string(),
            Span::dummy(),
        )]
    })
}
