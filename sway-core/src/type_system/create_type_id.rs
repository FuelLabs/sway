use crate::type_system::TypeId;

pub(crate) trait CreateTypeId {
    fn create_type_id(&self) -> TypeId;
}
