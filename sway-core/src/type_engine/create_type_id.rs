use super::TypeId;

pub(crate) trait CreateTypeId {
    fn create_type_id(&self) -> TypeId;
}
