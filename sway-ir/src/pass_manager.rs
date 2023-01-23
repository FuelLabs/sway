use crate::{Context, Function, IrError};
use downcast_rs::{impl_downcast, Downcast};
use std::collections::{hash_map, HashMap};

/// Result of an analysis. Specific result must be downcasted to.
pub trait AnalysisResultT: Downcast {}
impl_downcast!(AnalysisResultT);

/// A pass over the IR that can possibly modify it.
/// Name serves as unique identifier across all passes.
pub struct TransformPass {
    pub name: &'static str,
    pub descr: &'static str,
    pub run: fn(&mut Context, &Function) -> Result<bool, IrError>,
}

pub type AnalysisResult = Box<dyn AnalysisResultT>;

/// An analysis pass, producing an analysis result.
/// Name serves as unique identifier across all passes.
pub struct AnalysisPass {
    pub name: &'static str,
    pub descr: &'static str,
    pub run: fn(&mut Context, &Function) -> Result<AnalysisResult, IrError>,
}

pub enum Pass {
    AnalysisPass(AnalysisPass),
    TransformPass(TransformPass),
}

impl Pass {
    pub fn get_name(&self) -> &'static str {
        match self {
            Pass::AnalysisPass(ap) => ap.name,
            Pass::TransformPass(tp) => tp.name,
        }
    }
    pub fn get_descr(&self) -> &'static str {
        match self {
            Pass::AnalysisPass(ap) => ap.descr,
            Pass::TransformPass(tp) => tp.descr,
        }
    }
}

#[derive(Default)]
pub struct PassManager {
    passes: HashMap<&'static str, Pass>,
}

impl PassManager {
    /// Register a pass. Should be called only once for each pass.
    pub fn register(&mut self, pass: Pass) {
        match self.passes.entry(pass.get_name()) {
            hash_map::Entry::Occupied(_) => {
                panic!("Trying to register an already registered pass");
            }
            hash_map::Entry::Vacant(entry) => {
                entry.insert(pass);
            }
        }
    }

    /// Run the passes specified in `config`.
    pub fn run(&self, ir: &mut Context, config: &PMConfig) -> Result<(), IrError> {
        for pass in &config.to_run {
            dbg!(pass);
            match self.passes.get(pass.as_str()).expect("Unregistered pass") {
                Pass::AnalysisPass(_) => todo!(),
                Pass::TransformPass(tp) => {
                    for m in ir.module_iter() {
                        for f in m.function_iter(ir) {
                            (tp.run)(ir, &f)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Is `name` a registered pass?
    pub fn is_registered(&self, name: &str) -> bool {
        self.passes.contains_key(name)
    }

    pub fn help_text(&self) -> String {
        let summary = self
            .passes
            .iter()
            .map(|(name, pass)| format!("  {name:16} - {}", pass.get_descr()))
            .collect::<Vec<_>>()
            .join("\n");

        format!("Valid pass names are:\n\n{summary}",)
    }
}

/// Configuration for the pass manager to run passes.
pub struct PMConfig {
    pub to_run: Vec<String>,
}
