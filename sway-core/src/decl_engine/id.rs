use crate::decl_engine::*;

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

impl From<&DeclRef> for DeclId {
    fn from(value: &DeclRef) -> Self {
        value.id
    }
}
