use crate::{
    language::{HasModule, HasModuleId, HasSubmodules, ModName, Visibility},
    namespace::ModulePath,
    transform, Engines,
};

use super::{ParseModuleId, ParseTree};
use sway_error::{
    error::CompileError,
    handler::{ErrorEmitted, Handler},
};
use sway_types::Span;

pub type ModuleHash = u64;
pub type ModuleEvaluationOrder = Vec<ParseModuleId>;

/// A module and its submodules in the form of a tree.
#[derive(Debug, Clone)]
pub struct ParseModule {
    pub id: ParseModuleId,
    /// Parent module id or `None` if its a root module.
    pub parent: Option<ParseModuleId>,
    /// The content of this module in the form of a `ParseTree`.
    pub tree: ParseTree,
    /// Submodules introduced within this module using the `dep` syntax in order of declaration.
    pub submodules: Vec<(ModName, ParseSubmodule)>,
    pub attributes: transform::AttributesMap,
    /// The span of the module kind.
    pub module_kind_span: Span,
    /// Evaluation order for the submodules
    pub module_eval_order: ModuleEvaluationOrder,
    /// an empty span at the beginning of the file containing the module
    pub span: Span,
    /// an hash used for caching the module
    pub hash: ModuleHash,
    pub name: Option<String>,
}

impl ParseModule {
    /// Lookup the submodule at the given path.
    pub fn lookup_submodule(
        &self,
        handler: &Handler,
        engines: &Engines,
        path: &ModulePath,
    ) -> Result<ParseModuleId, ErrorEmitted> {
        let pme = engines.pme();
        let mut module_id = self.id;
        for ident in path.iter() {
            let module_arc = pme.get(&module_id);
            let module = module_arc.read().unwrap();
            match module.submodules.iter().find(|(name, _)| name == ident) {
                Some((_name, submod)) => module_id = submod.module,
                None => {
                    return Err(handler.emit_err(CompileError::Internal(
                        "Cannot find submodule",
                        Span::dummy(),
                    )))
                }
            }
        }
        Ok(module_id)
    }
}

impl HasModuleId for ParseModule {
    fn module_id(&self) -> ParseModuleId {
        self.id
    }
}

/// A library module that was declared as a `mod` of another module.
///
/// Only submodules are guaranteed to be a `library`.
#[derive(Debug, Clone)]
pub struct ParseSubmodule {
    pub module: ParseModuleId,
    pub mod_name_span: Span,
    pub visibility: Visibility,
}

impl HasModuleId for ParseSubmodule {
    fn module_id(&self) -> ParseModuleId {
        self.module
    }
}

impl HasModule<ParseModuleId> for ParseSubmodule {
    fn module(&self) -> &ParseModuleId {
        &self.module
    }
}

impl HasSubmodules<ParseSubmodule> for ParseModule {
    fn submodules(&self) -> &[(ModName, ParseSubmodule)] {
        &self.submodules
    }
}
