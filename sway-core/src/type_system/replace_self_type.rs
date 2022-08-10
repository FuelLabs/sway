use super::TypeId;

/// replace any instances of `TypeInfo::SelfType` with a provided [TypeId] `self_type`.
pub(crate) trait ReplaceSelfType {
    fn replace_self_type(&mut self, self_type: TypeId);
}
