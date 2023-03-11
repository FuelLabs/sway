use crate::{engine_threading::Engines, language::ty};

pub(crate) trait ReplaceFunctionImplementingType {
    fn replace_implementing_type(
        &mut self,
        engines: Engines<'_>,
        implementing_type: ty::TyDeclaration,
    );
}
