use crate::{type_system::TypeId, Engines};

pub(crate) trait CreateTypeId {
    fn create_type_id(&self, engines: Engines<'_>) -> TypeId;
}
