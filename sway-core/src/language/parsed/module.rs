use crate::{
    language::{HasModule, HasModuleId, HasSubmodules, ModName, Visibility},
    transform,
};

use super::{ParseModuleId, ParseTree};
use sway_types::Span;

pub type ModuleHash = u64;
pub type ModuleEvaluationOrder = Vec<ModName>;

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

impl HasModuleId for ParseModule
{
    fn module_id(&self) -> ParseModuleId {
        return self.id
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

impl HasModuleId for ParseSubmodule
{
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
