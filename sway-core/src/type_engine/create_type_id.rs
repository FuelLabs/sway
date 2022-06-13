use crate::type_engine::TypeId;

pub(crate) trait CreateTypeId {
    fn create_type_id(&self) -> TypeId;
}
