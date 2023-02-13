use crate::{
    create_const_combine_pass, create_dce_pass, create_dom_fronts_pass, create_dominators_pass,
    create_func_dce_pass, create_inline_in_main_pass, create_inline_in_non_predicate_pass,
    create_inline_in_predicate_pass, create_mem2reg_pass, create_postorder_pass,
    create_simplify_cfg_pass, Context, Function, IrError, Module,
};
use downcast_rs::{impl_downcast, Downcast};
use rustc_hash::FxHashMap;
use std::{
    any::{type_name, TypeId},
    collections::hash_map,
};

/// Result of an analysis. Specific result must be downcasted to.
pub trait AnalysisResultT: Downcast {}
impl_downcast!(AnalysisResultT);
pub type AnalysisResult = Box<dyn AnalysisResultT>;

/// Program scope over which a pass executes.
pub trait PassScope {
    fn get_arena_idx(&self) -> generational_arena::Index;
}
impl PassScope for Module {
    fn get_arena_idx(&self) -> generational_arena::Index {
        self.0
    }
}
impl PassScope for Function {
    fn get_arena_idx(&self) -> generational_arena::Index {
        self.0
    }
}

/// Is a pass an Analysis or a Transformation over the IR?
pub enum PassMutability<S: PassScope> {
    /// An analysis pass, producing an analysis result.
    Analysis(fn(&Context, analyses: &AnalysisResults, S) -> Result<AnalysisResult, IrError>),
    /// A pass over the IR that can possibly modify it.
    Transform(fn(&mut Context, analyses: &AnalysisResults, S) -> Result<bool, IrError>),
}

/// A concrete version of [PassScope].
pub enum ScopedPass {
    ModulePass(PassMutability<Module>),
    FunctionPass(PassMutability<Function>),
}

/// An analysis or transformation pass.
pub struct Pass {
    /// Pass identifier.
    pub name: &'static str,
    /// A short description.
    pub descr: &'static str,
    /// Other passes that this pass depends on.
    pub deps: Vec<&'static str>,
    /// The executor.
    ///
    pub runner: ScopedPass,
}

impl Pass {
    pub fn is_analysis(&self) -> bool {
        match &self.runner {
            ScopedPass::ModulePass(pm) => matches!(pm, PassMutability::Analysis(_)),
            ScopedPass::FunctionPass(pm) => matches!(pm, PassMutability::Analysis(_)),
        }
    }
    pub fn is_transform(&self) -> bool {
        !self.is_analysis()
    }
}

#[derive(Default)]
pub struct AnalysisResults {
    // Hash from (AnalysisResultT, (PassScope, Scope Identity)) to an actual result.
    results: FxHashMap<(TypeId, (TypeId, generational_arena::Index)), AnalysisResult>,
    name_typid_map: FxHashMap<&'static str, TypeId>,
}

impl AnalysisResults {
    /// Get the results of an analysis.
    /// Example analyses.get_analysis_result::<DomTreeAnalysis>(foo).
    pub fn get_analysis_result<T: AnalysisResultT, S: PassScope + 'static>(&self, scope: S) -> &T {
        self.results
            .get(&(
                TypeId::of::<T>(),
                (TypeId::of::<S>(), scope.get_arena_idx()),
            ))
            .unwrap_or_else(|| {
                panic!(
                    "Internal error. Analysis result {} unavailable for {} with idx {:?}",
                    type_name::<T>(),
                    type_name::<S>(),
                    scope.get_arena_idx()
                )
            })
            .downcast_ref()
            .expect("AnalysisResult: Incorrect type")
    }

    /// Is an analysis result available at the given scope?
    fn is_analysis_result_available<S: PassScope + 'static>(
        &self,
        name: &'static str,
        scope: S,
    ) -> bool {
        self.name_typid_map
            .get(name)
            .and_then(|result_typeid| {
                self.results
                    .get(&(*result_typeid, (TypeId::of::<S>(), scope.get_arena_idx())))
            })
            .is_some()
    }

    /// Add a new result.
    fn add_result<S: PassScope + 'static>(
        &mut self,
        name: &'static str,
        scope: S,
        result: AnalysisResult,
    ) {
        let result_typeid = (*result).type_id();
        self.results.insert(
            (result_typeid, (TypeId::of::<S>(), scope.get_arena_idx())),
            result,
        );
        self.name_typid_map.insert(name, result_typeid);
    }

    /// Invalidate all results at a given scope.
    fn invalidate_all_results_at_scope<S: PassScope + 'static>(&mut self, scope: S) {
        self.results
            .retain(|(_result_typeid, (scope_typeid, scope_idx)), _v| {
                (*scope_typeid, *scope_idx) != (TypeId::of::<S>(), scope.get_arena_idx())
            });
    }
}

#[derive(Default)]
pub struct PassManager {
    passes: FxHashMap<&'static str, Pass>,
    analyses: AnalysisResults,
}

impl PassManager {
    /// Register a pass. Should be called only once for each pass.
    pub fn register(&mut self, pass: Pass) -> &'static str {
        let pass_name = pass.name;
        match self.passes.entry(pass.name) {
            hash_map::Entry::Occupied(_) => {
                panic!("Trying to register an already registered pass");
            }
            hash_map::Entry::Vacant(entry) => {
                entry.insert(pass);
            }
        }
        pass_name
    }

    /// Run the passes specified in `config`.
    pub fn run(&mut self, ir: &mut Context, config: &PassManagerConfig) -> Result<bool, IrError> {
        let mut modified = false;
        let mut worklist: Vec<_> = config
            .to_run
            .iter()
            .rev()
            .map(|pass| self.passes.get(pass).expect("Unregistered pass"))
            .collect();
        while !worklist.is_empty() {
            // We clone because worklist may be modified later.
            let deps = worklist.last().unwrap().deps.clone();
            let mut unresolved_dep = false;
            // Check if all deps are satisfied
            for dep in deps {
                let dep_t = self.passes.get(dep).expect("Unregistered pass");
                for m in ir.module_iter() {
                    match &dep_t.runner {
                        ScopedPass::ModulePass(_) => {
                            if !self.analyses.is_analysis_result_available(dep_t.name, m) {
                                unresolved_dep = true;
                                worklist.push(dep_t);
                            }
                        }
                        ScopedPass::FunctionPass(_) => {
                            for f in m.function_iter(ir) {
                                if !self.analyses.is_analysis_result_available(dep_t.name, f) {
                                    unresolved_dep = true;
                                    worklist.push(dep_t);
                                    // If the analysis is unavailable for even one function,
                                    // we add it to worklist. Adding it once is sufficient.
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            if unresolved_dep {
                // New deps added. Start over.
                continue;
            }
            let pass_t = worklist.last().unwrap();
            for m in ir.module_iter() {
                match &pass_t.runner {
                    ScopedPass::ModulePass(mp) => match mp {
                        PassMutability::Analysis(analysis) => {
                            if !self.analyses.is_analysis_result_available(pass_t.name, m) {
                                let result = analysis(ir, &self.analyses, m)?;
                                self.analyses.add_result(pass_t.name, m, result);
                            }
                        }
                        PassMutability::Transform(transform) => {
                            if transform(ir, &self.analyses, m)? {
                                self.analyses.invalidate_all_results_at_scope(m);
                                for f in m.function_iter(ir) {
                                    self.analyses.invalidate_all_results_at_scope(f);
                                }
                                modified = true;
                            }
                        }
                    },
                    ScopedPass::FunctionPass(fp) => {
                        for f in m.function_iter(ir) {
                            match fp {
                                PassMutability::Analysis(analysis) => {
                                    if !self.analyses.is_analysis_result_available(pass_t.name, f) {
                                        let result = analysis(ir, &self.analyses, f)?;
                                        self.analyses.add_result(pass_t.name, f, result);
                                    }
                                }
                                PassMutability::Transform(transform) => {
                                    if transform(ir, &self.analyses, f)? {
                                        self.analyses.invalidate_all_results_at_scope(f);
                                        modified = true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            worklist.pop();
        }
        Ok(modified)
    }

    /// Get reference to a registered pass.
    pub fn lookup_registered_pass(&self, name: &str) -> Option<&Pass> {
        self.passes.get(name)
    }

    pub fn help_text(&self) -> String {
        let summary = self
            .passes
            .iter()
            .map(|(name, pass)| format!("  {name:16} - {}", pass.descr))
            .collect::<Vec<_>>()
            .join("\n");

        format!("Valid pass names are:\n\n{summary}",)
    }
}

/// Configuration for the pass manager to run passes.
pub struct PassManagerConfig {
    pub to_run: Vec<&'static str>,
}

/// A convenience utility to register known passes.
pub fn register_known_passes(pm: &mut PassManager) {
    // Analysis passes.
    pm.register(create_postorder_pass());
    pm.register(create_dominators_pass());
    pm.register(create_dom_fronts_pass());
    // Optimization passes.
    pm.register(create_mem2reg_pass());
    pm.register(create_inline_in_predicate_pass());
    pm.register(create_inline_in_non_predicate_pass());
    pm.register(create_inline_in_main_pass());
    pm.register(create_const_combine_pass());
    pm.register(create_simplify_cfg_pass());
    pm.register(create_func_dce_pass());
    pm.register(create_dce_pass());
}
