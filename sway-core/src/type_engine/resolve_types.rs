use crate::{
    namespace::{Path, Root},
    semantic_analysis::EnforceTypeArguments,
    CompileResult, TypeArgument,
};

pub(crate) trait ResolveTypes {
    fn resolve_types(
        &mut self,
        type_arguments: Vec<TypeArgument>,
        enforce_type_arguments: EnforceTypeArguments,
        namespace: &mut Root,
        module_path: &Path,
    ) -> CompileResult<()>;
}
