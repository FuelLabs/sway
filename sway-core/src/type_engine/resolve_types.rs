use crate::{
    namespace::{Path, Root},
    semantic_analysis::EnforceTypeArguments,
    CompileResult, TypeArgument,
};

use super::TypeId;

pub(crate) trait ResolveTypes {
    fn resolve_type_with_self(
        &mut self,
        type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        self_type: TypeId,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()>;

    fn resolve_type_without_self(
        &mut self,
        type_arguments: Vec<TypeArgument>,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()>;
}
