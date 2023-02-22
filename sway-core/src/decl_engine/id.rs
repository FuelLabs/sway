use crate::{decl_engine::*, engine_threading::*, type_system::*};

/// An ID used to refer to an item in the [DeclEngine](super::decl_engine::DeclEngine)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct DeclId(usize);

impl DeclId {
    pub(crate) fn new(id: usize) -> DeclId {
        DeclId(id)
    }

    pub(crate) fn replace_id(&mut self, index: DeclId) {
        self.0 = index.0;
    }
}

impl std::ops::Deref for DeclId {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(clippy::from_over_into)]
impl Into<usize> for DeclId {
    fn into(self) -> usize {
        self.0
    }
}

impl From<&DeclId> for DeclId {
    fn from(value: &DeclId) -> Self {
        DeclId::new(value.0)
    }
}

impl From<&mut DeclId> for DeclId {
    fn from(value: &mut DeclId) -> Self {
        DeclId::new(value.0)
    }
}

impl From<&DeclRef> for DeclId {
    fn from(value: &DeclRef) -> Self {
        value.id
    }
}

impl From<&mut DeclRef> for DeclId {
    fn from(value: &mut DeclRef) -> Self {
        value.id
    }
}

impl SubstTypes for DeclId {
    fn subst_inner(&mut self, type_mapping: &TypeSubstMap, engines: Engines<'_>) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.subst(type_mapping, engines);
        decl_engine.replace(self, decl);
    }
}

impl ReplaceSelfType for DeclId {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId) {
        let decl_engine = engines.de();
        let mut decl = decl_engine.get(self);
        decl.replace_self_type(engines, self_type);
        decl_engine.replace(self, decl);
    }
}
