//! This module handles the process of iterating through the typed AST and finishing the type
//! check unification step.

use sway_error::handler::{ErrorEmitted, Handler};

use super::TypeCheckContext;
use crate::{Engines, TypeId};

// A simple context that is used to finish type checking.
pub struct TypeCheckUnificationContext<'eng, 'ctx> {
    pub(crate) _engines: &'eng Engines,
    pub(crate) type_check_ctx: TypeCheckContext<'ctx>,
    pub(crate) type_id: Option<TypeId>,
}

impl<'eng, 'ctx> TypeCheckUnificationContext<'eng, 'ctx> {
    pub fn new(engines: &'eng Engines, type_check_ctx: TypeCheckContext<'ctx>) -> Self {
        Self {
            _engines: engines,
            type_check_ctx,
            type_id: None,
        }
    }
}

pub(crate) trait TypeCheckUnification {
    fn type_check_unify(
        &mut self,
        handler: &Handler,
        ctx: &mut TypeCheckUnificationContext,
    ) -> Result<(), ErrorEmitted>;
}
