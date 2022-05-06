use crate::type_engine::TypeId;

pub(crate) trait CreateTypeId {
    fn type_id(&self) -> TypeId;
}
