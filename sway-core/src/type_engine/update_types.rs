use crate::{error::*, namespace::*, type_engine::*};

pub(crate) trait UpdateTypes {
    fn update_types_with_self(
        &mut self,
        type_mapping: &TypeMapping,
        namespace: &mut Namespace,
        self_type: TypeId,
    ) -> CompileResult<()>;

    fn update_types_without_self(
        &mut self,
        type_mapping: &TypeMapping,
        namespace: &mut Namespace,
    ) -> CompileResult<()>;
}
