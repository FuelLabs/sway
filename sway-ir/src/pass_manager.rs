use crate::{Context, Function, IrError};
use std::collections::HashMap;

pub trait NamedPass {
    fn name() -> &'static str;
    fn descr() -> &'static str;
    fn run(ir: &mut Context) -> Result<bool, IrError>;

    fn run_on_all_fns<F: FnMut(&mut Context, &Function) -> Result<bool, IrError>>(
        ir: &mut Context,
        mut run_on_fn: F,
    ) -> Result<bool, IrError> {
        let funcs = ir
            .module_iter()
            .flat_map(|module| module.function_iter(ir))
            .collect::<Vec<_>>();
        let mut modified = false;
        for func in funcs {
            if run_on_fn(ir, &func)? {
                modified = true;
            }
        }
        Ok(modified)
    }
}

pub type NamePassPair = (&'static str, fn(&mut Context) -> Result<bool, IrError>);

#[derive(Default)]
pub struct PassManager {
    passes: HashMap<&'static str, NamePassPair>,
}

impl PassManager {
    pub fn register<T: NamedPass>(&mut self) {
        self.passes.insert(T::name(), (T::descr(), T::run));
    }

    pub fn run(&self, name: &str, ir: &mut Context) -> Result<bool, IrError> {
        self.passes.get(name).expect("Unknown pass name!").1(ir)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.passes.contains_key(name)
    }

    pub fn help_text(&self) -> String {
        let summary = self
            .passes
            .iter()
            .map(|(name, (descr, _))| format!("  {name:16} - {descr}"))
            .collect::<Vec<_>>()
            .join("\n");

        format!("Valid pass names are:\n\n{summary}",)
    }
}
