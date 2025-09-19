use crate::{
    create_arg_demotion_pass, create_arg_pointee_mutability_tagger_pass, create_ccp_pass,
    create_const_demotion_pass, create_const_folding_pass, create_cse_pass, create_dce_pass,
    create_dom_fronts_pass, create_dominators_pass, create_escaped_symbols_pass,
    create_fn_dedup_debug_profile_pass, create_fn_dedup_release_profile_pass,
    create_fn_inline_pass, create_globals_dce_pass, create_mem2reg_pass, create_memcpyopt_pass,
    create_misc_demotion_pass, create_module_printer_pass, create_module_verifier_pass,
    create_postorder_pass, create_ret_demotion_pass, create_simplify_cfg_pass, create_sroa_pass,
    Context, Function, IrError, Module, ARG_DEMOTION_NAME, ARG_POINTEE_MUTABILITY_TAGGER_NAME,
    CCP_NAME, CONST_DEMOTION_NAME, CONST_FOLDING_NAME, CSE_NAME, DCE_NAME,
    FN_DEDUP_DEBUG_PROFILE_NAME, FN_DEDUP_RELEASE_PROFILE_NAME, FN_INLINE_NAME, GLOBALS_DCE_NAME,
    MEM2REG_NAME, MEMCPYOPT_NAME, MISC_DEMOTION_NAME, RET_DEMOTION_NAME, SIMPLIFY_CFG_NAME,
    SROA_NAME,
};
use downcast_rs::{impl_downcast, Downcast};
use rustc_hash::FxHashMap;
use std::{
    any::{type_name, TypeId},
    collections::{hash_map, HashSet},
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
#[derive(Clone)]
pub enum PassMutability<S: PassScope> {
    /// An analysis pass, producing an analysis result.
    Analysis(fn(&Context, analyses: &AnalysisResults, S) -> Result<AnalysisResult, IrError>),
    /// A pass over the IR that can possibly modify it.
    Transform(fn(&mut Context, analyses: &AnalysisResults, S) -> Result<bool, IrError>),
}

/// A concrete version of [PassScope].
#[derive(Clone)]
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

    pub fn is_module_pass(&self) -> bool {
        matches!(self.runner, ScopedPass::ModulePass(_))
    }

    pub fn is_function_pass(&self) -> bool {
        matches!(self.runner, ScopedPass::FunctionPass(_))
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

/// Options for printing [Pass]es in case of running them with printing requested.
///
/// Note that states of IR can always be printed by injecting the module printer pass
/// and just running the passes. That approach however offers less control over the
/// printing. E.g., requiring the printing to happen only if the previous passes
/// modified the IR cannot be done by simply injecting a module printer.
#[derive(Debug)]
pub struct PrintPassesOpts {
    pub initial: bool,
    pub r#final: bool,
    pub modified_only: bool,
    pub passes: HashSet<String>,
}

#[derive(Default)]
pub struct PassManager {
    passes: FxHashMap<&'static str, Pass>,
    analyses: AnalysisResults,
}

impl PassManager {
    pub const OPTIMIZATION_PASSES: [&'static str; 14] = [
        FN_INLINE_NAME,
        SIMPLIFY_CFG_NAME,
        SROA_NAME,
        DCE_NAME,
        GLOBALS_DCE_NAME,
        FN_DEDUP_RELEASE_PROFILE_NAME,
        FN_DEDUP_DEBUG_PROFILE_NAME,
        MEM2REG_NAME,
        MEMCPYOPT_NAME,
        CONST_FOLDING_NAME,
        ARG_DEMOTION_NAME,
        CONST_DEMOTION_NAME,
        RET_DEMOTION_NAME,
        MISC_DEMOTION_NAME,
    ];

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
                if pass.is_function_pass() && dep_t.is_module_pass() {
                    panic!(
                        "Function pass {} cannot depend on module pass {}",
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

        fn run_module_pass(
            pm: &mut PassManager,
            ir: &mut Context,
            pass: &'static str,
            module: Module,
        ) -> Result<bool, IrError> {
            let mut modified = false;
            let pass_t = pm.passes.get(pass).expect("Unregistered pass");
            for dep in pass_t.deps.clone() {
                let dep_t = pm.passes.get(dep).expect("Unregistered dependent pass");
                // If pass registration allows transformations as dependents, we could remove this I guess.
                assert!(dep_t.is_analysis());
                match dep_t.runner {
                    ScopedPass::ModulePass(_) => {
                        if !pm.analyses.is_analysis_result_available(dep, module) {
                            run_module_pass(pm, ir, dep, module)?;
                        }
                    }
                    ScopedPass::FunctionPass(_) => {
                        for f in module.function_iter(ir) {
                            if !pm.analyses.is_analysis_result_available(dep, f) {
                                run_function_pass(pm, ir, dep, f)?;
                            }
                        }
                    }
                }
            }

            // Get the pass again to satisfy the borrow checker.
            let pass_t = pm.passes.get(pass).expect("Unregistered pass");
            let ScopedPass::ModulePass(mp) = pass_t.runner.clone() else {
                panic!("Expected a module pass");
            };
            match mp {
                PassMutability::Analysis(analysis) => {
                    let result = analysis(ir, &pm.analyses, module)?;
                    pm.analyses.add_result(pass, module, result);
                }
                PassMutability::Transform(transform) => {
                    if transform(ir, &pm.analyses, module)? {
                        pm.analyses.invalidate_all_results_at_scope(module);
                        for f in module.function_iter(ir) {
                            pm.analyses.invalidate_all_results_at_scope(f);
                        }
                        modified = true;
                    }
                }
            }

            Ok(modified)
        }

        fn run_function_pass(
            pm: &mut PassManager,
            ir: &mut Context,
            pass: &'static str,
            function: Function,
        ) -> Result<bool, IrError> {
            let mut modified = false;
            let pass_t = pm.passes.get(pass).expect("Unregistered pass");
            for dep in pass_t.deps.clone() {
                let dep_t = pm.passes.get(dep).expect("Unregistered dependent pass");
                // If pass registration allows transformations as dependents, we could remove this I guess.
                assert!(dep_t.is_analysis());
                match dep_t.runner {
                    ScopedPass::ModulePass(_) => {
                        panic!(
                            "Function pass {} cannot depend on module pass {}",
                            pass, dep
                        )
                    }
                    ScopedPass::FunctionPass(_) => {
                        if !pm.analyses.is_analysis_result_available(dep, function) {
                            run_function_pass(pm, ir, dep, function)?;
                        };
                    }
                }
            }

            // Get the pass again to satisfy the borrow checker.
            let pass_t = pm.passes.get(pass).expect("Unregistered pass");
            let ScopedPass::FunctionPass(fp) = pass_t.runner.clone() else {
                panic!("Expected a function pass");
            };
            match fp {
                PassMutability::Analysis(analysis) => {
                    let result = analysis(ir, &pm.analyses, function)?;
                    pm.analyses.add_result(pass, function, result);
                }
                PassMutability::Transform(transform) => {
                    if transform(ir, &pm.analyses, function)? {
                        pm.analyses.invalidate_all_results_at_scope(function);
                        modified = true;
                    }
                }
            }

            Ok(modified)
        }

        for m in ir.module_iter() {
            let pass_t = self.passes.get(pass).expect("Unregistered pass");
            let pass_runner = pass_t.runner.clone();
            match pass_runner {
                ScopedPass::ModulePass(_) => {
                    modified |= run_module_pass(self, ir, pass, m)?;
                }
                ScopedPass::FunctionPass(_) => {
                    for f in m.function_iter(ir) {
                        modified |= run_function_pass(self, ir, pass, f)?;
                    }
                }
            }
        }
        Ok(modified)
    }

    /// Run the `passes` and return true if the `passes` modify the initial `ir`.
    pub fn run(&mut self, ir: &mut Context, passes: &PassGroup) -> Result<bool, IrError> {
        let mut modified = false;
        for pass in passes.flatten_pass_group() {
            modified |= self.actually_run(ir, pass)?;
        }
        Ok(modified)
    }

    /// Run the `passes` and return true if the `passes` modify the initial `ir`.
    /// The IR states are printed according to the printing options provided in `print_opts`.
    pub fn run_with_print(
        &mut self,
        ir: &mut Context,
        passes: &PassGroup,
        print_opts: &PrintPassesOpts,
    ) -> Result<bool, IrError> {
        // Empty IRs are result of compiling dependencies. We don't want to print those.
        fn ir_is_empty(ir: &Context) -> bool {
            ir.functions.is_empty()
                && ir.blocks.is_empty()
                && ir.values.is_empty()
                && ir.local_vars.is_empty()
        }

        fn print_ir_after_pass(ir: &Context, pass: &Pass) {
            if !ir_is_empty(ir) {
                println!("// IR: [{}] {}", pass.name, pass.descr);
                println!("{ir}");
            }
        }

        fn print_initial_or_final_ir(ir: &Context, initial_or_final: &'static str) {
            if !ir_is_empty(ir) {
                println!("// IR: {initial_or_final}");
                println!("{ir}");
            }
        }

        if print_opts.initial {
            print_initial_or_final_ir(ir, "Initial");
        }

        let mut modified = false;
        for pass in passes.flatten_pass_group() {
            let modified_in_pass = self.actually_run(ir, pass)?;

            if print_opts.passes.contains(pass) && (!print_opts.modified_only || modified_in_pass) {
                print_ir_after_pass(ir, self.lookup_registered_pass(pass).unwrap());
            }

            modified |= modified_in_pass;
        }

        if print_opts.r#final {
            print_initial_or_final_ir(ir, "Final");
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
    pm.register(create_arg_pointee_mutability_tagger_pass());
    pm.register(create_fn_dedup_release_profile_pass());
    pm.register(create_fn_dedup_debug_profile_pass());
    pm.register(create_mem2reg_pass());
    pm.register(create_sroa_pass());
    pm.register(create_fn_inline_pass());
    pm.register(create_const_folding_pass());
    pm.register(create_ccp_pass());
    pm.register(create_simplify_cfg_pass());
    pm.register(create_globals_dce_pass());
    pm.register(create_dce_pass());
    pm.register(create_cse_pass());
    pm.register(create_arg_demotion_pass());
    pm.register(create_const_demotion_pass());
    pm.register(create_ret_demotion_pass());
    pm.register(create_misc_demotion_pass());
    pm.register(create_memcpyopt_pass());
}

pub fn create_o1_pass_group() -> PassGroup {
    // Create a create_ccp_passo specify which passes we want to run now.
    let mut o1 = PassGroup::default();
    // Configure to run our passes.
    o1.append_pass(MEM2REG_NAME);
    o1.append_pass(FN_DEDUP_RELEASE_PROFILE_NAME);
    o1.append_pass(FN_INLINE_NAME);
    o1.append_pass(ARG_POINTEE_MUTABILITY_TAGGER_NAME);
    o1.append_pass(SIMPLIFY_CFG_NAME);
    o1.append_pass(GLOBALS_DCE_NAME);
    o1.append_pass(DCE_NAME);
    o1.append_pass(FN_INLINE_NAME);
    o1.append_pass(ARG_POINTEE_MUTABILITY_TAGGER_NAME);
    o1.append_pass(CCP_NAME);
    o1.append_pass(CONST_FOLDING_NAME);
    o1.append_pass(SIMPLIFY_CFG_NAME);
    o1.append_pass(CSE_NAME);
    o1.append_pass(CONST_FOLDING_NAME);
    o1.append_pass(SIMPLIFY_CFG_NAME);
    o1.append_pass(GLOBALS_DCE_NAME);
    o1.append_pass(DCE_NAME);
    o1.append_pass(FN_DEDUP_RELEASE_PROFILE_NAME);

    o1
}

/// Utility to insert a pass after every pass in the given group `pg`.
/// It preserves the `pg` group's structure. This means if `pg` has subgroups
/// and those have subgroups, the resulting [PassGroup] will have the
/// same subgroups, but with the `pass` inserted after every pass in every
/// subgroup, as well as all passes outside of any groups.
pub fn insert_after_each(pg: PassGroup, pass: &'static str) -> PassGroup {
    fn insert_after_each_rec(pg: PassGroup, pass: &'static str) -> Vec<PassOrGroup> {
        pg.0.into_iter()
            .flat_map(|p_o_g| match p_o_g {
                PassOrGroup::Group(group) => vec![PassOrGroup::Group(PassGroup(
                    insert_after_each_rec(group, pass),
                ))],
                PassOrGroup::Pass(_) => vec![p_o_g, PassOrGroup::Pass(pass)],
            })
            .collect()
    }

    PassGroup(insert_after_each_rec(pg, pass))
}
