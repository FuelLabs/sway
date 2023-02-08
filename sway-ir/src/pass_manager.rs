use crate::{Context, Function, IrError, Module};
use downcast_rs::{impl_downcast, Downcast};
use std::{
    any::{type_name, TypeId},
    collections::{hash_map, HashMap},
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
    Analysis(fn(&mut Context, analyses: &AnalysisResults, S) -> Result<AnalysisResult, IrError>),
    /// A pass over the IR that can possibly modify it.
    Transform(fn(&mut Context, analyses: &AnalysisResults, S) -> Result<bool, IrError>),
}

/// A concrete version of [PassScope].
pub enum ScopedPass {
    ModulePass(PassMutability<Module>),
    FunctionPass(PassMutability<Function>),
}

pub struct Pass {
    pub name: &'static str,
    pub descr: &'static str,
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
    results: HashMap<(TypeId, (TypeId, generational_arena::Index)), AnalysisResult>,
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
            .unwrap()
    }

    /// Add a new result.
    pub fn add_result<S: PassScope + 'static>(&mut self, scope: S, result: AnalysisResult) {
        self.results.insert(
            (
                (*result).type_id(),
                (TypeId::of::<S>(), scope.get_arena_idx()),
            ),
            result,
        );
    }
}

#[derive(Default)]
pub struct PassManager {
    passes: HashMap<&'static str, Pass>,
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
        for pass in &config.to_run {
            let pass_t = self.passes.get(pass.as_str()).expect("Unregistered pass");
            for m in ir.module_iter() {
                match &pass_t.runner {
                    ScopedPass::ModulePass(mp) => match mp {
                        PassMutability::Analysis(analysis) => {
                            let result = analysis(ir, &self.analyses, m)?;
                            self.analyses.add_result(m, result);
                        }
                        PassMutability::Transform(transform) => {
                            modified |= transform(ir, &self.analyses, m)?;
                        }
                    },
                    ScopedPass::FunctionPass(fp) => {
                        for f in m.function_iter(ir) {
                            match fp {
                                PassMutability::Analysis(analysis) => {
                                    let result = analysis(ir, &self.analyses, f)?;
                                    self.analyses.add_result(f, result);
                                }
                                PassMutability::Transform(transform) => {
                                    modified |= transform(ir, &self.analyses, f)?;
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(modified)
    }

    /// Is `name` a registered pass?
    pub fn is_registered(&self, name: &str) -> bool {
        self.passes.contains_key(name)
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
    pub to_run: Vec<String>,
}
