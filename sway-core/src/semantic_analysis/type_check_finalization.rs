//! This module handles the process of iterating through the typed AST and finishing the type
//! checking step, for type checking steps that need to look at information that was computed
//! from the initial type checked tree.

use sway_error::handler::{ErrorEmitted, Handler};

use crate::Engines;

use super::TypeCheckContext;

// A simple context that is used to finish type checking.
pub struct TypeCheckFinalizationContext<'eng, 'ctx> {
    pub(crate) engines: &'eng Engines,
    #[allow(dead_code)]
    pub(crate) type_check_ctx: TypeCheckContext<'ctx>,
}

impl<'eng, 'ctx> TypeCheckFinalizationContext<'eng, 'ctx> {
    pub fn new(engines: &'eng Engines, type_check_ctx: TypeCheckContext<'ctx>) -> Self {
        Self {
            engines,
            type_check_ctx,
        }
    }
}

pub(crate) trait TypeCheckFinalization {
    fn type_check_finalize(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckFinalizationContext,
    ) -> Result<(), ErrorEmitted>;
}
