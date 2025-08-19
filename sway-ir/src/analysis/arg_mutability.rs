//! For every function argument that is a pointer, determine if that function
//! may mutate the corresponding pointee.

use rustc_hash::FxHashMap;

use crate::{
    build_call_graph, callee_first_order, AnalysisResult, AnalysisResultT, AnalysisResults,
    Context, Function, IrError, Module, Pass, PassMutability, ScopedPass,
};

#[derive(Debug, Clone)]
/// The mutability of a pointer function argument's pointee.
pub enum ArgPointeeMutability {
    Immutable,
    Mutable,
    NotAPointer,
}

// The dominator tree is represented by mapping each Block to its DomTreeNode.
#[derive(Default)]
pub struct ArgPointeeMutabilityResult(FxHashMap<Function, Vec<ArgPointeeMutability>>);
impl AnalysisResultT for ArgPointeeMutabilityResult {}

pub const ARG_POINTEE_MUTABILITY_NAME: &str = "arg_pointee_mutability";

pub fn create_arg_pointee_mutability_pass() -> Pass {
    Pass {
        name: ARG_POINTEE_MUTABILITY_NAME,
        descr: "Analyze the mutability of function argument pointees",
        deps: vec![],
        runner: ScopedPass::ModulePass(PassMutability::Analysis(
            compute_arg_pointee_mutability_pass,
        )),
    }
}

impl ArgPointeeMutabilityResult {
    /// Get the mutability of the pointee for a function argument.
    /// Panics on invalid function or argument index.
    pub fn get_mutability(&self, function: Function, arg_index: usize) -> ArgPointeeMutability {
        self.0.get(&function).unwrap()[arg_index].clone()
    }
}

pub fn compute_arg_pointee_mutability_pass(
    context: &Context,
    _: &AnalysisResults,
    module: Module,
) -> Result<AnalysisResult, IrError> {
    Ok(Box::new(compute_arg_pointee_mutability(context, module)))
}

/// Compute the mutability of function argument pointees.
pub fn compute_arg_pointee_mutability(
    context: &Context,
    module: Module,
) -> ArgPointeeMutabilityResult {
    let cg = build_call_graph(context, &context.modules.get(module.0).unwrap().functions);
    let _callee_first = callee_first_order(&cg);

    todo!()
}
