use crate::engine_threading::Engines;

use super::TypeId;

/// replace any instances of `TypeInfo::SelfType` with a provided [TypeId] `self_type`.
pub(crate) trait ReplaceSelfType {
    fn replace_self_type(&mut self, engines: Engines<'_>, self_type: TypeId);
}
