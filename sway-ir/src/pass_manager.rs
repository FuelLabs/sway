use crate::{
    create_arg_demotion_pass, create_const_combine_pass, create_const_demotion_pass,
    create_dce_pass, create_dom_fronts_pass, create_dominators_pass, create_escaped_symbols_pass,
    create_fn_dedup_debug_profile_pass, create_fn_dedup_release_profile_pass, create_func_dce_pass,
    create_inline_in_main_pass, create_inline_in_module_pass, create_mem2reg_pass,
    create_memcpyopt_pass, create_misc_demotion_pass, create_module_printer_pass,
    create_module_verifier_pass, create_postorder_pass, create_ret_demotion_pass,
    create_simplify_cfg_pass, create_sroa_pass, Context, Function, IrError, Module,
    CONSTCOMBINE_NAME, DCE_NAME, FNDEDUP_RELEASE_PROFILE_NAME, FUNC_DCE_NAME, INLINE_MODULE_NAME,
    MEM2REG_NAME, SIMPLIFYCFG_NAME,
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
    fn get_arena_idx(&self) -> slotmap::DefaultKey;
}
impl PassScope for Module {
    fn get_arena_idx(&self) -> slotmap::DefaultKey {
        self.0
    }
}
impl PassScope for Function {
    fn get_arena_idx(&self) -> slotmap::DefaultKey {
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
    results: FxHashMap<(TypeId, (TypeId, slotmap::DefaultKey)), AnalysisResult>,
    name_typeid_map: FxHashMap<&'static str, TypeId>,
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
        self.name_typeid_map
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
        self.name_typeid_map.insert(name, result_typeid);
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
        for dep in &pass.deps {
            if let Some(dep_t) = self.lookup_registered_pass(dep) {
                if dep_t.is_transform() {
                    panic!(
                        "Pass {} cannot depend on a transformation pass {}",
                        pass.name, dep
                    );
                }
            } else {
                panic!(
                    "Pass {} depends on a (yet) unregistered pass {}",
                    pass.name, dep
                );
            }
        }
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

    fn actually_run(&mut self, ir: &mut Context, pass: &'static str) -> Result<bool, IrError> {
        let mut modified = false;
        let pass_t = self.passes.get(pass).expect("Unregistered pass");

        // Run passes that this depends on.
        for dep in pass_t.deps.clone() {
            self.actually_run(ir, dep)?;
        }

        // To please the borrow checker, get current pass again.
        let pass_t = self.passes.get(pass).expect("Unregistered pass");

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
                                    self.analyses.invalidate_all_results_at_scope(m);
                                    modified = true;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    /// Run the passes specified in `config`.
    pub fn run(&mut self, ir: &mut Context, passes: &PassGroup) -> Result<bool, IrError> {
        let mut modified = false;
        for pass in passes.flatten_pass_group() {
            modified |= self.actually_run(ir, pass)?;
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

/// A group of passes.
/// Can contain sub-groups.
#[derive(Default)]
pub struct PassGroup(Vec<PassOrGroup>);

/// An individual pass, or a group (with possible subgroup) of passes.
pub enum PassOrGroup {
    Pass(&'static str),
    Group(PassGroup),
}

impl PassGroup {
    // Flatten a group of passes into an ordered list.
    fn flatten_pass_group(&self) -> Vec<&'static str> {
        let mut output = Vec::<&str>::new();
        fn inner(output: &mut Vec<&str>, input: &PassGroup) {
            for pass_or_group in &input.0 {
                match pass_or_group {
                    PassOrGroup::Pass(pass) => output.push(pass),
                    PassOrGroup::Group(pg) => inner(output, pg),
                }
            }
        }
        inner(&mut output, self);
        output
    }

    /// Append a pass to this group.
    pub fn append_pass(&mut self, pass: &'static str) {
        self.0.push(PassOrGroup::Pass(pass));
    }

    /// Append a pass group.
    pub fn append_group(&mut self, group: PassGroup) {
        self.0.push(PassOrGroup::Group(group));
    }
}

/// A convenience utility to register known passes.
pub fn register_known_passes(pm: &mut PassManager) {
    // Analysis passes.
    pm.register(create_postorder_pass());
    pm.register(create_dominators_pass());
    pm.register(create_dom_fronts_pass());
    pm.register(create_escaped_symbols_pass());
    pm.register(create_module_printer_pass());
    pm.register(create_module_verifier_pass());
    // Optimization passes.
    pm.register(create_fn_dedup_release_profile_pass());
    pm.register(create_fn_dedup_debug_profile_pass());
    pm.register(create_mem2reg_pass());
    pm.register(create_sroa_pass());
    pm.register(create_inline_in_module_pass());
    pm.register(create_inline_in_main_pass());
    pm.register(create_const_combine_pass());
    pm.register(create_simplify_cfg_pass());
    pm.register(create_func_dce_pass());
    pm.register(create_dce_pass());
    pm.register(create_arg_demotion_pass());
    pm.register(create_const_demotion_pass());
    pm.register(create_ret_demotion_pass());
    pm.register(create_misc_demotion_pass());
    pm.register(create_memcpyopt_pass());
}

pub fn create_o1_pass_group() -> PassGroup {
    // Create a configuration to specify which passes we want to run now.
    let mut o1 = PassGroup::default();
    // Configure to run our passes.
    o1.append_pass(MEM2REG_NAME);
    o1.append_pass(INLINE_MODULE_NAME);
    o1.append_pass(FNDEDUP_RELEASE_PROFILE_NAME);
    o1.append_pass(CONSTCOMBINE_NAME);
    o1.append_pass(SIMPLIFYCFG_NAME);
    o1.append_pass(CONSTCOMBINE_NAME);
    o1.append_pass(SIMPLIFYCFG_NAME);
    o1.append_pass(FUNC_DCE_NAME);
    o1.append_pass(DCE_NAME);

    o1
}

/// Utility to insert a pass after every pass in the given group
pub fn insert_after_each(pg: PassGroup, pass: &'static str) -> PassGroup {
    PassGroup(
        pg.0.into_iter()
            .flat_map(|p_o_g| vec![p_o_g, PassOrGroup::Pass(pass)])
            .collect(),
    )
}
