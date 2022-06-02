use crate::{error::*, namespace::*, type_engine::*};

pub(crate) trait UpdateTypes {
    fn update_types(
        &mut self,
        type_mapping: &TypeMapping,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> CompileResult<()>;
}
