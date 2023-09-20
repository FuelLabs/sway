use crate::{
    language::ty::TyTypeAliasDecl,
    semantic_analysis::{TypeCheckFinalization, TypeCheckFinalizationContext},
};
use sway_error::handler::{ErrorEmitted, Handler};

impl TypeCheckFinalization for TyTypeAliasDecl {
    fn type_check_finalize(
        &mut self,
        _handler: &Handler,
        _ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted> {
        Ok(())
    }
}
