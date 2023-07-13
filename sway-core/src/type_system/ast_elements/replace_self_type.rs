use crate::{engine_threading::Engines, type_system::priv_prelude::*};

/// Replace any instances of `TypeInfo::SelfType` with a provided [TypeId]
/// `self_type`.
pub trait ReplaceSelfType {
    fn replace_self_type(&mut self, engines: &Engines, self_type: TypeId);
}
