use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

use crate::{
    language::{ty, CallPath, QualifiedCallPath},
    namespace::ModulePath,
    EnforceTypeArguments, Engines, Namespace, TypeId,
};

pub trait TypeResolver {
    /// Resolve the type of the given [TypeId], replacing any instances of
    /// [TypeInfo::Custom] with either a monomorphized struct, monomorphized
    /// enum, or a reference to a type parameter.
    #[allow(clippy::too_many_arguments)]
    fn resolve(
        &self,
        handler: &Handler,
        engines: &Engines,
        namespace: &Namespace,
        type_id: TypeId,
        span: &Span,
        enforce_type_arguments: EnforceTypeArguments,
        type_info_prefix: Option<&ModulePath>,
        mod_path: &ModulePath,
        self_type: Option<TypeId>,
    ) -> Result<TypeId, ErrorEmitted>;

    fn resolve_qualified_call_path(
        &self,
        handler: &Handler,
        engines: &Engines,
        namespace: &Namespace,
        mod_path: &ModulePath,
        qualified_call_path: &QualifiedCallPath,
    ) -> Result<ty::TyDecl, ErrorEmitted>;

    fn resolve_call_path(
        &self,
        handler: &Handler,
        engines: &Engines,
        namespace: &Namespace,
        mod_path: &ModulePath,
        call_path: &CallPath,
    ) -> Result<ty::TyDecl, ErrorEmitted>;
}
