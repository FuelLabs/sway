use sway_types::{Ident, SourceEngine};

use crate::{
    language::{parsed::ImplSelfOrTrait, ty::TyTraitDecl, CallPathType},
    namespace::Module,
};

impl TyTraitDecl {
    pub(crate) fn is_marker_trait(&self) -> bool {
        assert!(
            matches!(self.call_path.callpath_type, CallPathType::Full),
            "call paths of trait declarations must always be full paths"
        );

        is_std_marker_module_path(&self.call_path.prefixes)
    }
}

impl Module {
    pub(crate) fn is_std_marker_module(&self) -> bool {
        is_std_marker_module_path(self.mod_path())
    }
}

impl ImplSelfOrTrait {
    pub(crate) fn is_autogenerated(&self, source_engine: &SourceEngine) -> bool {
        source_engine
            .is_span_in_autogenerated(&self.block_span)
            .unwrap_or(false)
    }
}

fn is_std_marker_module_path(path: &[Ident]) -> bool {
    path.len() == 2 && path[0].as_str() == "std" && path[1].as_str() == "marker"
}
