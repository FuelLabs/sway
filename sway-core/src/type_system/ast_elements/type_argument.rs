use crate::{engine_threading::*, language::CallPathTree, type_system::priv_prelude::*};
use std::{cmp::Ordering, fmt, hash::Hasher};
use sway_types::{Span, Spanned};

#[derive(Debug, Clone)]
pub struct TypeArgument {
    pub type_id: TypeId,
    pub initial_type_id: TypeId,
    pub span: Span,
    pub call_path_tree: Option<CallPathTree>,
}

impl From<TypeId> for TypeArgument {
    fn from(type_id: TypeId) -> Self {
        TypeArgument {
            type_id,
            initial_type_id: type_id,
            span: Span::dummy(),
            call_path_tree: None,
        }
    }
}

impl Spanned for TypeArgument {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl HashWithEngines for TypeArgument {
    fn hash<H: Hasher>(&self, state: &mut H, engines: Engines<'_>) {
        let TypeArgument {
            type_id,
            // these fields are not hashed because they aren't relevant/a
            // reliable source of obj v. obj distinction
            initial_type_id: _,
            span: _,
            call_path_tree: _,
        } = self;
        let type_engine = engines.te();
        type_engine.get(*type_id).hash(state, engines);
    }
}

impl EqWithEngines for TypeArgument {}
impl PartialEqWithEngines for TypeArgument {
    fn eq(&self, other: &Self, engines: Engines<'_>) -> bool {
        let type_engine = engines.te();
        type_engine
            .get(self.type_id)
            .eq(&type_engine.get(other.type_id), engines)
    }
}

impl OrdWithEngines for TypeArgument {
    fn cmp(&self, other: &Self, engines: Engines<'_>) -> Ordering {
        let TypeArgument {
            type_id: lti,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            initial_type_id: _,
            span: _,
            call_path_tree: _,
        } = self;
        let TypeArgument {
            type_id: rti,
            // these fields are not compared because they aren't relevant/a
            // reliable source of obj v. obj distinction
            initial_type_id: _,
            span: _,
            call_path_tree: _,
        } = other;
        engines.te().get(*lti).cmp(&engines.te().get(*rti), engines)
    }
}

impl DebugWithEngines for TypeArgument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>, engines: Engines<'_>) -> fmt::Result {
        write!(f, "{:?}", engines.help_out(engines.te().get(self.type_id)))
    }
}

impl From<&TypeParameter> for TypeArgument {
    fn from(type_param: &TypeParameter) -> Self {
        TypeArgument {
            type_id: type_param.type_id,
            initial_type_id: type_param.initial_type_id,
            span: type_param.name_ident.span(),
            call_path_tree: None,
        }
    }
}

impl ReplaceSelfType for TypeArgument {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        self.type_id.replace_self_type(engines, self_type);
    }
}

impl SubstTypes for TypeArgument {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        self.type_id.subst(type_mapping, engines);
    }
}
