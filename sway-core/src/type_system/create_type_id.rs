use super::{TypeEngine, TypeId};

pub(crate) trait CreateTypeId {
    fn create_type_id(&self, type_engine: &TypeEngine) -> TypeId;
}
