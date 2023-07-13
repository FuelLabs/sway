use crate::{type_system::priv_prelude::*, Engines};

pub(crate) trait CreateTypeId {
    fn create_type_id(&self, engines: &Engines) -> TypeId;
}
