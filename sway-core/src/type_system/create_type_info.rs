use crate::type_system::TypeId;

pub(crate) trait CreateTypeInfo {
    fn create_type_id(&self) -> TypeId;
}
