use std::collections::HashMap;

use sway_error::handler::{ErrorEmitted, Handler};
use sway_types::Span;

use crate::{
    engine_threading::Engines,
    language::ty::{self, TyDecl},
    semantic_analysis::TypeCheckContext,
};

use super::DeclMapping;

pub trait ReplaceDecls {
    fn replace_decls_inner(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
        already_replaced: &mut HashMap<(usize, std::any::TypeId), (usize, Span)>,
    ) -> Result<(), ErrorEmitted>;

    fn replace_decls(
        &mut self,
        decl_mapping: &DeclMapping,
        handler: &Handler,
        ctx: &mut TypeCheckContext,
        already_replaced: &mut HashMap<(usize, std::any::TypeId), (usize, Span)>,
    ) -> Result<(), ErrorEmitted> {
        if !decl_mapping.is_empty() {
            self.replace_decls_inner(decl_mapping, handler, ctx, already_replaced)?;
        }

        Ok(())
    }
}

pub(crate) trait ReplaceFunctionImplementingType {
    fn replace_implementing_type(&mut self, engines: &Engines, implementing_type: ty::TyDecl);
}

pub(crate) trait UpdateConstantExpression {
    fn update_constant_expression(&mut self, engines: &Engines, implementing_type: &TyDecl);
}
